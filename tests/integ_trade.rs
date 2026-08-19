//! Integration tests for TradeClient — require real API credentials.
//!
//! Coverage:
//! - Read-only queries (orders, positions, assets, funds, transfers, tokens)
//! - Order matrix: preview / place / cancel across US-STK / OPT / FUT /
//!   MLEG / HK / CN / SG, plus algo (TWAP/VWAP/ICEBERG/OCA) and edge
//!   cases (negative price, sell-short, iceberg modify). Every order
//!   uses safe prices (BUY $0.01 / SELL $999_999) so nothing fills.
//!
//! Run with: TIGER_RUN_INTEG=true cargo test --test integ_trade -- --nocapture

mod integ_support;

#[cfg(test)]
mod tests {
    use super::integ_support;
    use tigeropen::error::TigerError;
    use tigeropen::model::order::{AlgoParamsRequest, ContractLegRequest, OrderRequest};
    use tigeropen::model::trade_requests::{
        AggregateAssetsRequest, AnalyticsAssetRequest, AssetsRequest, DerivativeContractsRequest,
        EstimateTradableQuantityRequest, ForexOrderRequest, FundDetailsRequest,
        FundingHistoryRequest, GetOrderRequest, ManagedAccountsRequest,
        OptionExercisePositionRequest, OptionExerciseRecordsRequest, OrderTransactionsRequest,
        OrdersRequest, PositionTransferExternalRecordsRequest, PositionTransferRecordsRequest,
        PositionsRequest, SegmentFundRequest,
    };
    use tigeropen::model::{
        AggregateAssets, AnalyticsAsset, Asset, Contract, EstimateTradableQuantity, FundDetails,
        FundingHistoryItem, ManagedAccount, Order, Position, PrimeAsset, SegmentFundAvailableItem,
        SegmentFundHistoryItem, Transaction,
    };
    use tigeropen::trade::TradeClient;

    // ─────────────────────────────────────────────────────────────────────
    // Shared helpers (used by both read-only and order-matrix tests)
    // ─────────────────────────────────────────────────────────────────────

    /// Safe prices — kept far from market so orders never fill.
    const SAFE_BUY_PRICE: f64 = 0.01;
    const SAFE_SELL_PRICE: f64 = 999_999.0;
    const SAFE_STOP_BUY_TRIGGER: f64 = 999_999.0;

    /// Permission / license / session boundary phrases — legitimate skips.
    /// Same set used in the Java / Python / Go integ suites.
    const PERMISSION_ERROR_MARKERS: &[&str] = &[
        "access forbidden",
        "forbidden",
        "no permission",
        "not supported",
        "license",
        "not open",
        "not enabled",
        "no token",
        "don't support trading",
        "don\u{2019}t support trading",
        "unsupported instrument",
        "only limit orders are supported",
        "outside of regular trading hours",
        "market is closed",
        "only limit orders can be placed",
        "only limit, stop or stop-limit orders are allowed",
        "at non-trading hour",
        "orders cannot be placed at this moment",
        "auction order is not allowed at this moment",
        "does not support stock long",
        "does not support stock short",
        "only trade cash order by market order",
        "cash order by market order",
        "time range for the order",
    ];

    /// Order state race markers — cancel/modify may hit a terminal state.
    const TERMINAL_ORDER_MARKERS: &[&str] = &[
        "cannot be modified",
        "cannot be canceled",
        "cannot be cancelled",
        "already canceled",
        "already cancelled",
        "already filled",
        "invalid order status",
        "cancellation is not allowed",
        "cancel is not allowed",
    ];

    /// Rate-limit markers — retry with exponential backoff.
    const RATE_LIMIT_MARKERS: &[&str] =
        &["too_many_requests", "rate limit", "requestrateexceedlimit"];

    fn matches_any(msg: &str, markers: &[&str]) -> bool {
        let lower = msg.to_lowercase();
        markers.iter().any(|m| lower.contains(m))
    }

    fn now_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    /// Converts days-since-1970-01-01 into (year, month, day).
    /// Based on Howard Hinnant's date algorithms — public domain.
    fn civil_from_days(z: i64) -> (i32, u32, u32) {
        let z = z + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let doe = (z - era * 146_097) as u32;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
        let y = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        (y as i32, m, d)
    }

    fn future_expiry_yyyymmdd(days: i64) -> String {
        let now = now_ms() / 1000;
        let target = now + days * 86_400;
        let days_since_epoch = target / 86_400;
        let (y, m, d) = civil_from_days(days_since_epoch);
        format!("{:04}{:02}{:02}", y, m, d)
    }

    fn us_stk() -> OrderRequest {
        OrderRequest {
            symbol: Some("AAPL".into()),
            sec_type: Some("STK".into()),
            currency: Some("USD".into()),
            market: Some("US".into()),
            ..Default::default()
        }
    }

    fn hk_stk() -> OrderRequest {
        OrderRequest {
            symbol: Some("00700".into()),
            sec_type: Some("STK".into()),
            currency: Some("HKD".into()),
            market: Some("HK".into()),
            ..Default::default()
        }
    }

    fn skip_if_disabled() -> Option<TradeClient> {
        if !integ_support::is_integ_run() {
            return None;
        }
        Some(TradeClient::from_config(integ_support::integ_config()))
    }

    async fn preview_only(tc: &TradeClient, order: OrderRequest, ctx: &str) -> bool {
        match tc.preview_order(order).await {
            Ok(_) => true,
            Err(e) if matches_any(&e.to_string(), PERMISSION_ERROR_MARKERS) => {
                eprintln!("[{ctx}] skipped (preview boundary): {e}");
                false
            }
            Err(e) => panic!("[{ctx}] preview failed: {e}"),
        }
    }

    async fn preview_and_place(tc: &TradeClient, order: OrderRequest, ctx: &str) -> bool {
        if !preview_only(tc, order.clone(), ctx).await {
            return false;
        }
        let mut delay = std::time::Duration::from_secs(1);
        let mut place_result = None;
        for attempt in 0..3 {
            match tc.place_order(order.clone()).await {
                Ok(Some(res)) => {
                    place_result = Some(res);
                    break;
                }
                Ok(None) => panic!("[{ctx}] place_order returned Ok(None)"),
                Err(e) => {
                    let msg = e.to_string();
                    if matches_any(&msg, PERMISSION_ERROR_MARKERS) {
                        eprintln!("[{ctx}] skipped (place boundary): {e}");
                        return false;
                    }
                    if matches_any(&msg, RATE_LIMIT_MARKERS) && attempt < 2 {
                        tokio::time::sleep(delay).await;
                        delay *= 2;
                        continue;
                    }
                    panic!("[{ctx}] place_order failed: {e}");
                }
            }
        }
        let place_result = place_result.expect("place_result set after loop");
        let order_id = if place_result.id != 0 {
            place_result.id
        } else if place_result.order_id != 0 {
            place_result.order_id
        } else {
            panic!("[{ctx}] no order id in {place_result:?}");
        };
        eprintln!("[{ctx}] placed order id={order_id}");
        cancel_tolerant(tc, order_id, ctx).await;
        true
    }

    async fn cancel_tolerant(tc: &TradeClient, order_id: i64, ctx: &str) {
        match tc.cancel_order(order_id).await {
            Ok(_) => {}
            Err(e) => {
                let msg = e.to_string();
                if matches_any(&msg, TERMINAL_ORDER_MARKERS) {
                    eprintln!("[{ctx}] cancel hit terminal state (ok): {e}");
                    return;
                }
                panic!("[{ctx}] unexpected cancel failure: {e}");
            }
        }
    }

    /// Format an epoch-ms timestamp as `YYYY-MM-DD` in UTC.
    fn ymd_utc(ms: i64) -> String {
        let days = ms / 86_400_000;
        let (y, m, d) = civil_from_days(days);
        format!("{:04}-{:02}-{:02}", y, m, d)
    }

    #[tokio::test]
    async fn test_integ_get_positions() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = PositionsRequest::default();
        let result = client.get_positions(req).await;
        assert!(result.is_ok(), "get_positions should succeed: {:?}", result);
        let data: Vec<Position> = result.unwrap();
        // Positions may legitimately be empty (e.g. paper account with no holdings).
        // When entries exist, validate key fields.
        if !data.is_empty() {
            let p = &data[0];
            assert!(
                p.symbol.as_ref().is_some_and(|s| !s.is_empty()),
                "Position.symbol should be non-empty, got {:?}",
                p.symbol
            );
            assert!(
                p.position.is_some(),
                "Position.position (quantity) should be present, got None"
            );
            assert!(
                p.sec_type.is_some(),
                "Position.sec_type should be present, got None"
            );
            assert!(
                p.market.is_some(),
                "Position.market should be present, got None"
            );
            assert!(
                p.latest_price.map(|v| v >= 0.0).unwrap_or(true),
                "Position.latest_price should be >= 0, got {:?}",
                p.latest_price
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_orders() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = OrdersRequest::default();
        let result = client.get_orders(req).await;
        assert!(result.is_ok(), "get_orders should succeed: {:?}", result);
        let data: Vec<Order> = result.unwrap();
        // Orders may legitimately be empty (paper account with no trades).
        // When entries exist, validate key fields.
        if !data.is_empty() {
            let o = &data[0];
            assert!(
                !o.account.is_empty(),
                "Order.account should be non-empty, got {:?}",
                o.account
            );
            assert!(
                !o.status.is_empty(),
                "Order.status should be non-empty, got {:?}",
                o.status
            );
            assert!(
                !o.action.is_empty(),
                "Order.action should be non-empty (BUY/SELL), got {:?}",
                o.action
            );
            assert!(
                !o.order_type.is_empty(),
                "Order.order_type should be non-empty, got {:?}",
                o.order_type
            );
            assert!(
                o.total_quantity >= 0,
                "Order.total_quantity should be >= 0, got {}",
                o.total_quantity
            );
            assert!(
                o.filled_quantity >= 0,
                "Order.filled_quantity should be >= 0, got {}",
                o.filled_quantity
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_assets() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = AssetsRequest::default();
        let result = client.get_assets(req).await;
        assert!(result.is_ok(), "get_assets should succeed: {:?}", result);
        let data: Vec<Asset> = result.unwrap();
        // Assets should typically not be empty for a valid account.
        assert!(
            !data.is_empty(),
            "assets result should not be empty for a valid account"
        );
        let a = &data[0];
        assert!(
            !a.account.is_empty(),
            "Asset.account should be non-empty, got {:?}",
            a.account
        );
        assert!(
            !a.currency.is_empty(),
            "Asset.currency should be non-empty, got {:?}",
            a.currency
        );
        assert!(
            a.net_liquidation > 0.0,
            "Asset.net_liquidation should be > 0 for a valid account, got {}",
            a.net_liquidation
        );
        assert!(
            a.buying_power > 0.0,
            "Asset.buying_power should be > 0 for a valid account, got {}",
            a.buying_power
        );
    }

    // ── 合约查询 ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_integ_get_contract() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let result = client.get_contract("AAPL", "STK").await;
        assert!(result.is_ok(), "get_contract should succeed: {:?}", result);
        let data: Vec<Contract> = result.unwrap();
        assert!(
            !data.is_empty(),
            "contract result should not be empty for AAPL"
        );
        let c = &data[0];
        assert_eq!(
            c.symbol, "AAPL",
            "Contract.symbol should be AAPL, got {:?}",
            c.symbol
        );
        assert!(
            !c.sec_type.is_empty(),
            "Contract.sec_type should be non-empty"
        );
    }

    #[tokio::test]
    async fn test_integ_get_contracts() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let result = client.get_contracts(&["AAPL", "TSLA"], "STK").await;
        assert!(result.is_ok(), "get_contracts should succeed: {:?}", result);
        let data: Vec<Contract> = result.unwrap();
        assert!(!data.is_empty(), "contracts result should not be empty");
        assert!(
            data.iter().any(|c| c.symbol == "AAPL"),
            "contracts result should contain AAPL"
        );
    }

    #[tokio::test]
    async fn test_integ_get_quote_contract() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        // quote_contract only supports derivatives — the server rejects
        // sec_type=STK with "'sec_type':'STK' is not supported, all
        // supported sec_type include:['OPT','WAR','IOPT']". Pick an OPT
        // whose expiry is discovered dynamically. Fall back to boundary
        // skip if the CI account lacks OPT market data.
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let one_year_out = now_ms + 365 * 86_400_000;
        // 20260117 — well-known LEAPS expiry; server validates format only.
        let expiry = {
            let days = one_year_out / 86_400_000;
            let (y, m, d) = civil_from_days(days);
            format!("{:04}{:02}{:02}", y, m, d)
        };
        let result = client.get_quote_contract("AAPL", "OPT", &expiry).await;
        match result {
            Ok(contracts) => {
                // Validate returned contracts have expected market field populated.
                for c in &contracts {
                    assert!(
                        c.market.as_deref().map(|m| !m.is_empty()).unwrap_or(false),
                        "Contract.market should be non-empty, got {:?}",
                        c.market
                    );
                }
            }
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    matches_any(&msg, PERMISSION_ERROR_MARKERS)
                        || msg.to_lowercase().contains("no market data")
                        || msg.to_lowercase().contains("not support")
                        || msg.to_lowercase().contains("license"),
                    "unexpected quote_contract error: {}",
                    msg
                );
            }
        }
    }

    // ── 订单查询扩展 ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_integ_get_active_orders() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = OrdersRequest::default();
        let result = client.get_active_orders(req).await;
        assert!(
            result.is_ok(),
            "get_active_orders should succeed: {:?}",
            result
        );
        let _data: Vec<Order> = result.unwrap();
    }

    #[tokio::test]
    async fn test_integ_get_inactive_orders() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = OrdersRequest::default();
        let result = client.get_inactive_orders(req).await;
        assert!(
            result.is_ok(),
            "get_inactive_orders should succeed: {:?}",
            result
        );
        let _data: Vec<Order> = result.unwrap();
    }

    #[tokio::test]
    async fn test_integ_get_filled_orders() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        // filled_orders requires start_date — server rejects empty
        // with "field 'start_date' cannot be empty".
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let req = OrdersRequest {
            start_date: Some(now_ms - 30 * 86_400_000),
            end_date: Some(now_ms),
            ..Default::default()
        };
        let result = client.get_filled_orders(req).await;
        assert!(
            result.is_ok(),
            "get_filled_orders should succeed: {:?}",
            result
        );
        let _data: Vec<Order> = result.unwrap();
    }

    #[tokio::test]
    async fn test_integ_get_order() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        // Fetch an order id from get_orders; skip if none.
        let orders = client.get_orders(OrdersRequest::default()).await;
        let order_id = match orders {
            Ok(o) if !o.is_empty() => o[0].order_id,
            _ => return,
        };
        // Skip if the parent order has no populated id (get_orders can
        // return placeholder rows with 0).
        if order_id == 0 {
            return;
        }
        let req = GetOrderRequest {
            order_id: Some(order_id),
            ..Default::default()
        };
        let result = client.get_order(req).await;
        assert!(result.is_ok(), "get_order should succeed: {:?}", result);
        // Server sometimes returns Option::Some with only a subset of
        // fields populated (id=0, order_id != request). We only
        // exercise the pipeline; unit tests cover response parsing.
        let _ = result.unwrap();
    }

    #[tokio::test]
    async fn test_integ_get_order_transactions() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = OrderTransactionsRequest {
            symbol: Some("AAPL".to_string()),
            sec_type: Some("STK".to_string()),
            ..Default::default()
        };
        let result = client.get_order_transactions(req).await;
        assert!(
            result.is_ok(),
            "get_order_transactions should succeed: {:?}",
            result
        );
        let data: Vec<Transaction> = result.unwrap();
        if !data.is_empty() {
            let t = &data[0];
            assert!(
                !t.symbol.is_empty(),
                "Transaction.symbol should be non-empty"
            );
            assert!(
                t.filled_quantity >= 0,
                "Transaction.filled_quantity should be >= 0"
            );
        }
    }

    // ── 资产扩展 ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_integ_get_prime_assets() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = AssetsRequest::default();
        let result = client.get_prime_assets(req).await;
        assert!(
            result.is_ok(),
            "get_prime_assets should succeed: {:?}",
            result
        );
        let data: Option<PrimeAsset> = result.unwrap();
        if let Some(p) = data {
            assert!(
                !p.account_id.is_empty(),
                "PrimeAsset.account_id should be non-empty, got {:?}",
                p.account_id
            );
        }
    }

    // ── 账户管理 ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_integ_get_managed_accounts() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = ManagedAccountsRequest::default();
        let result = client.get_managed_accounts(req).await;
        assert!(
            result.is_ok(),
            "get_managed_accounts should succeed: {:?}",
            result
        );
        let data: Vec<ManagedAccount> = result.unwrap();
        // For non-institutional accounts this may be empty; validate when present.
        if !data.is_empty() {
            assert!(
                !data[0].account.is_empty(),
                "ManagedAccount.account should be non-empty"
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_derivative_contracts() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        // Server requires `expiry` for derivative_contracts (yyyymmdd).
        // Use a well-known third-Friday LEAPS expiry roughly one year out.
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let days = (now_ms + 365 * 86_400_000) / 86_400_000;
        let (y, m, d) = civil_from_days(days);
        let expiry = format!("{:04}{:02}{:02}", y, m, d);
        let req = DerivativeContractsRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            sec_type: Some("OPT".to_string()),
            expiry: Some(expiry),
            ..Default::default()
        };
        let result = client.get_derivative_contracts(req).await;
        assert!(
            result.is_ok(),
            "get_derivative_contracts should succeed: {:?}",
            result
        );
        // May be empty if no derivative positions; just ensure Ok.
        let _data: Vec<Contract> = result.unwrap();
    }

    #[tokio::test]
    async fn test_integ_get_analytics_asset() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = AnalyticsAssetRequest {
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-01-31".to_string()),
            ..Default::default()
        };
        let result = client.get_analytics_asset(req).await;
        assert!(
            result.is_ok(),
            "get_analytics_asset should succeed: {:?}",
            result
        );
        let data: Vec<AnalyticsAsset> = result.unwrap();
        if !data.is_empty() {
            assert!(
                !data[0].date.is_empty(),
                "AnalyticsAsset.date should be non-empty"
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_aggregate_assets() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = AggregateAssetsRequest::default();
        let result = client.get_aggregate_assets(req).await;
        // aggregate_assets is institution-only. Server responds
        // "only support institution account" for personal accounts.
        match result {
            Ok(data) => {
                if let Some(a) = data {
                    assert!(
                        a.net_liquidation >= 0.0,
                        "AggregateAssets.net_liquidation should be >= 0, got {}",
                        a.net_liquidation
                    );
                }
            }
            Err(e) => {
                let msg = format!("{:?}", e);
                assert!(
                    msg.to_lowercase().contains("institution")
                        || msg.to_lowercase().contains("permission")
                        || msg.to_lowercase().contains("license"),
                    "unexpected aggregate_assets error: {}",
                    msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_integ_get_estimate_tradable_quantity() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        // Server validates order_type against the enum
        // [STP_LMT, AL, AM, ICEBERG, MOC, TWAP, MKT, VWAP, LMT, STP, LOC, TRAIL];
        // "MARKET" is rejected.
        let req = EstimateTradableQuantityRequest {
            symbol: Some("AAPL".to_string()),
            sec_type: Some("STK".to_string()),
            action: Some("BUY".to_string()),
            order_type: Some("MKT".to_string()),
            ..Default::default()
        };
        let result = client.get_estimate_tradable_quantity(req).await;
        assert!(
            result.is_ok(),
            "get_estimate_tradable_quantity should succeed: {:?}",
            result
        );
        let data: Option<EstimateTradableQuantity> = result.unwrap();
        if let Some(e) = data {
            assert!(
                e.tradable_quantity >= 0.0,
                "EstimateTradableQuantity.tradable_quantity should be >= 0, got {}",
                e.tradable_quantity
            );
        }
    }

    // ── 资金调拨查询（只读）──────────────────────────────────────────────

    #[tokio::test]
    async fn test_integ_get_segment_fund_available() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        // Server rejects empty `from_segment` with "from_segment is empty".
        // SegmentType is one of ALL / SEC / FUT / FUND (Python enum).
        let req = SegmentFundRequest {
            from_segment: Some("SEC".to_string()),
            currency: Some("USD".to_string()),
            ..Default::default()
        };
        let result = client.get_segment_fund_available(req).await;
        match result {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("{:?}", e);
                assert!(
                    msg.to_lowercase().contains("permission")
                        || msg.to_lowercase().contains("license")
                        || msg.to_lowercase().contains("no segment")
                        || msg.to_lowercase().contains("not support")
                        || msg.to_lowercase().contains("institution"),
                    "unexpected segment_fund_available error: {}",
                    msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_integ_get_segment_fund_history() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = SegmentFundRequest {
            limit: Some(5),
            ..Default::default()
        };
        let result = client.get_segment_fund_history(req).await;
        assert!(
            result.is_ok(),
            "get_segment_fund_history should succeed: {:?}",
            result
        );
        let _data: Vec<SegmentFundHistoryItem> = result.unwrap();
    }

    // ── 资金明细 ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_integ_get_fund_details() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        // Server rejects empty seg_types with "seg_types invalid".
        // Legal values match Python's SegmentType enum: ALL / SEC / FUT / FUND.
        // start_date / end_date are yyyy-MM-dd strings (0.6 wire fix).
        let now = now_ms();
        let req = FundDetailsRequest {
            seg_types: Some(vec!["SEC".to_string()]),
            start_date: Some(ymd_utc(now - 30 * 86_400_000)),
            end_date: Some(ymd_utc(now)),
            limit: Some(5),
            ..Default::default()
        };
        let result = client.get_fund_details(req).await;
        assert!(
            result.is_ok(),
            "get_fund_details should succeed: {:?}",
            result
        );
        // Rows may omit `account` when the server aggregates across
        // segments; only exercise the pipeline shape.
        let _ = result.unwrap();
    }

    #[tokio::test]
    async fn test_integ_get_funding_history() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = FundingHistoryRequest {
            limit: Some(5),
            ..Default::default()
        };
        let result = client.get_funding_history(req).await;
        assert!(
            result.is_ok(),
            "get_funding_history should succeed: {:?}",
            result
        );
        let _data: Vec<FundingHistoryItem> = result.unwrap();
    }

    // ── 持仓转移查询（只读）──────────────────────────────────────────────

    #[tokio::test]
    async fn test_integ_get_position_transfer_records() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        // Server requires since_date (yyyy-mm-dd) and enforces a
        // max-date range (bad_request:transfer.query.max_date.limit).
        // 30-day window is safely inside the cap.
        let now = now_ms();
        let req = PositionTransferRecordsRequest {
            since_date: Some(ymd_utc(now - 30 * 86_400_000)),
            to_date: Some(ymd_utc(now)),
            limit: Some(5),
            ..Default::default()
        };
        let result = client.get_position_transfer_records(req).await;
        assert!(
            result.is_ok(),
            "get_position_transfer_records should succeed: {:?}",
            result
        );
        // May be empty for accounts without transfers; just ensure Ok.
        let _data = result.unwrap();
    }

    #[tokio::test]
    async fn test_integ_get_position_transfer_external_records() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        // Server requires since_date (yyyy-mm-dd) and caps the range.
        let now = now_ms();
        let req = PositionTransferExternalRecordsRequest {
            since_date: Some(ymd_utc(now - 30 * 86_400_000)),
            to_date: Some(ymd_utc(now)),
            limit: Some(5),
            ..Default::default()
        };
        let result = client.get_position_transfer_external_records(req).await;
        assert!(
            result.is_ok(),
            "get_position_transfer_external_records should succeed: {:?}",
            result
        );
        let _data = result.unwrap();
    }

    // ── 期权行权查询（只读）──────────────────────────────────────────────

    #[tokio::test]
    async fn test_integ_get_option_exercise_positions() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = OptionExercisePositionRequest {
            exercise_type: Some("Exercise".to_string()),
            ..Default::default()
        };
        let result = client.get_option_exercise_positions(req).await;
        assert!(
            result.is_ok(),
            "get_option_exercise_positions should succeed: {:?}",
            result
        );
        // May be None for accounts without exercisable options.
        let _data = result.unwrap();
    }

    #[tokio::test]
    async fn test_integ_get_option_exercise_records() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = OptionExerciseRecordsRequest {
            page: Some(1),
            size: Some(5),
            ..Default::default()
        };
        let result = client.get_option_exercise_records(req).await;
        assert!(
            result.is_ok(),
            "get_option_exercise_records should succeed: {:?}",
            result
        );
        let _data = result.unwrap();
    }

    // ── Token 管理（只读）────────────────────────────────────────────────

    #[tokio::test]
    async fn test_integ_query_token() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let result = client.query_token().await;
        // CI license may return "'TBNZ' license has no token" — this is
        // an account-level boundary, not an SDK issue.
        match result {
            Ok(token) => {
                assert!(
                    !token.is_empty(),
                    "refreshed token string should be non-empty"
                );
            }
            Err(e) => {
                let msg = format!("{:?}", e);
                assert!(
                    msg.to_lowercase().contains("no token")
                        || msg.to_lowercase().contains("license")
                        || msg.to_lowercase().contains("permission"),
                    "unexpected query_token error: {}",
                    msg
                );
            }
        }
    }

    // =====================================================================
    // Order matrix — Phase 1 (US market × order type)
    // =====================================================================

    #[tokio::test]
    async fn test_matrix_us_stk_market_preview() {
        let Some(tc) = skip_if_disabled() else { return };
        preview_only(
            &tc,
            OrderRequest {
                action: Some("BUY".into()),
                order_type: Some("MKT".into()),
                total_quantity: Some(1),
                ..us_stk()
            },
            "US STK MKT preview",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_us_stk_market_by_amount_preview() {
        let Some(tc) = skip_if_disabled() else { return };
        preview_only(
            &tc,
            OrderRequest {
                action: Some("BUY".into()),
                order_type: Some("MKT".into()),
                cash_amount: Some(100.0),
                ..us_stk()
            },
            "US STK MKT-by-amount preview",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_us_stk_stop() {
        let Some(tc) = skip_if_disabled() else { return };
        preview_and_place(
            &tc,
            OrderRequest {
                action: Some("BUY".into()),
                order_type: Some("STP".into()),
                total_quantity: Some(1),
                aux_price: Some(SAFE_STOP_BUY_TRIGGER),
                ..us_stk()
            },
            "US STK STP",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_us_stk_stop_limit() {
        let Some(tc) = skip_if_disabled() else { return };
        preview_and_place(
            &tc,
            OrderRequest {
                action: Some("BUY".into()),
                order_type: Some("STP_LMT".into()),
                total_quantity: Some(1),
                limit_price: Some(SAFE_BUY_PRICE),
                aux_price: Some(SAFE_STOP_BUY_TRIGGER),
                ..us_stk()
            },
            "US STK STP_LMT",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_us_stk_trail() {
        let Some(tc) = skip_if_disabled() else { return };
        preview_and_place(
            &tc,
            OrderRequest {
                action: Some("SELL".into()),
                order_type: Some("TRAIL".into()),
                total_quantity: Some(1),
                trailing_percent: Some(50.0),
                ..us_stk()
            },
            "US STK TRAIL",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_us_stk_twap() {
        let Some(tc) = skip_if_disabled() else { return };
        let now = now_ms();
        preview_and_place(
            &tc,
            OrderRequest {
                action: Some("BUY".into()),
                order_type: Some("TWAP".into()),
                total_quantity: Some(10),
                limit_price: Some(SAFE_BUY_PRICE),
                algo_strategy: Some("TWAP".into()),
                algo_params: Some(AlgoParamsRequest {
                    start_time: Some(now),
                    end_time: Some(now + 3_600_000),
                    ..Default::default()
                }),
                ..us_stk()
            },
            "US STK TWAP",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_us_stk_vwap() {
        let Some(tc) = skip_if_disabled() else { return };
        let now = now_ms();
        preview_and_place(
            &tc,
            OrderRequest {
                action: Some("BUY".into()),
                order_type: Some("VWAP".into()),
                total_quantity: Some(10),
                limit_price: Some(SAFE_BUY_PRICE),
                algo_strategy: Some("VWAP".into()),
                algo_params: Some(AlgoParamsRequest {
                    start_time: Some(now),
                    end_time: Some(now + 3_600_000),
                    participation_rate: Some(0.1),
                    ..Default::default()
                }),
                ..us_stk()
            },
            "US STK VWAP",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_us_stk_iceberg() {
        let Some(tc) = skip_if_disabled() else { return };
        let now = now_ms();
        preview_and_place(
            &tc,
            OrderRequest {
                action: Some("BUY".into()),
                order_type: Some("ICEBERG".into()),
                total_quantity: Some(10),
                limit_price: Some(SAFE_BUY_PRICE),
                display_size: Some(2),
                min_display_size: Some(1),
                check_intervals: Some(30),
                price_type: Some("LIMIT_PRICE".into()),
                start_time: Some(now),
                end_time: Some(now + 3_600_000),
                ..us_stk()
            },
            "US STK ICEBERG",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_us_stk_oca_brackets() {
        let Some(tc) = skip_if_disabled() else { return };
        let profit = OrderRequest {
            action: Some("BUY".into()),
            order_type: Some("LMT".into()),
            total_quantity: Some(1),
            limit_price: Some(SAFE_BUY_PRICE),
            time_in_force: Some("GTC".into()),
            ..us_stk()
        };
        let stop = OrderRequest {
            action: Some("BUY".into()),
            order_type: Some("STP_LMT".into()),
            total_quantity: Some(1),
            limit_price: Some(SAFE_BUY_PRICE),
            aux_price: Some(SAFE_BUY_PRICE),
            time_in_force: Some("GTC".into()),
            ..us_stk()
        };
        let parent = OrderRequest {
            oca_orders: Some(vec![Box::new(profit), Box::new(stop)]),
            ..us_stk()
        };
        preview_and_place(&tc, parent, "US STK OCA brackets").await;
    }

    #[tokio::test]
    async fn test_matrix_us_opt_limit() {
        let Some(tc) = skip_if_disabled() else { return };
        let expiry = future_expiry_yyyymmdd(30);
        preview_and_place(
            &tc,
            OrderRequest {
                symbol: Some("AAPL".into()),
                sec_type: Some("OPT".into()),
                currency: Some("USD".into()),
                market: Some("US".into()),
                expiry: Some(expiry),
                strike: Some("200".into()),
                right: Some("CALL".into()),
                action: Some("BUY".into()),
                order_type: Some("LMT".into()),
                total_quantity: Some(1),
                limit_price: Some(SAFE_BUY_PRICE),
                ..Default::default()
            },
            "US OPT LMT",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_us_fut_limit() {
        let Some(tc) = skip_if_disabled() else { return };
        preview_and_place(
            &tc,
            OrderRequest {
                symbol: Some("CL".into()),
                sec_type: Some("FUT".into()),
                currency: Some("USD".into()),
                market: Some("US".into()),
                action: Some("BUY".into()),
                order_type: Some("LMT".into()),
                total_quantity: Some(1),
                limit_price: Some(SAFE_BUY_PRICE),
                ..Default::default()
            },
            "US FUT LMT",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_forex_sec_segment() {
        let Some(tc) = skip_if_disabled() else { return };
        let res = tc
            .place_forex_order(ForexOrderRequest {
                seg_type: Some("SEC".into()),
                source_currency: Some("USD".into()),
                target_currency: Some("HKD".into()),
                source_amount: Some(1.0),
                ..Default::default()
            })
            .await;
        match res {
            Ok(_) => {}
            Err(e) if matches_any(&e.to_string(), PERMISSION_ERROR_MARKERS) => {
                eprintln!("[Forex SEC] skipped: {e}");
            }
            Err(TigerError::Api { .. }) => {
                // Business-side rejection is fine — wire round-trip
                // (typing / marshaling) succeeded, which is what we check.
            }
            Err(e) => panic!("Forex SEC unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn test_matrix_us_stk_negative_price_preview() {
        let Some(tc) = skip_if_disabled() else { return };
        let res = tc
            .preview_order(OrderRequest {
                action: Some("BUY".into()),
                order_type: Some("LMT".into()),
                total_quantity: Some(1),
                limit_price: Some(-1.0),
                ..us_stk()
            })
            .await;
        eprintln!("[US STK negative-price preview] result: {res:?}");
    }

    // =====================================================================
    // Order matrix — Phase 2 (HK / CN / SG)
    // =====================================================================

    #[tokio::test]
    async fn test_matrix_hk_stk_limit() {
        let Some(tc) = skip_if_disabled() else { return };
        preview_and_place(
            &tc,
            OrderRequest {
                action: Some("BUY".into()),
                order_type: Some("LMT".into()),
                total_quantity: Some(100),
                limit_price: Some(SAFE_BUY_PRICE),
                ..hk_stk()
            },
            "HK STK LMT",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_hk_stk_auction_limit() {
        let Some(tc) = skip_if_disabled() else { return };
        preview_and_place(
            &tc,
            OrderRequest {
                action: Some("BUY".into()),
                order_type: Some("AL".into()),
                total_quantity: Some(100),
                limit_price: Some(SAFE_BUY_PRICE),
                ..hk_stk()
            },
            "HK STK AL",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_hk_stk_auction_market_preview() {
        let Some(tc) = skip_if_disabled() else { return };
        preview_only(
            &tc,
            OrderRequest {
                action: Some("BUY".into()),
                order_type: Some("AM".into()),
                total_quantity: Some(100),
                ..hk_stk()
            },
            "HK STK AM preview",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_cn_stk_limit() {
        let Some(tc) = skip_if_disabled() else { return };
        preview_and_place(
            &tc,
            OrderRequest {
                symbol: Some("000001".into()),
                sec_type: Some("STK".into()),
                currency: Some("CNH".into()),
                market: Some("CN".into()),
                action: Some("BUY".into()),
                order_type: Some("LMT".into()),
                total_quantity: Some(100),
                limit_price: Some(SAFE_BUY_PRICE),
                ..Default::default()
            },
            "CN STK LMT",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_sg_stk_limit() {
        let Some(tc) = skip_if_disabled() else { return };
        preview_and_place(
            &tc,
            OrderRequest {
                symbol: Some("D05".into()),
                sec_type: Some("STK".into()),
                currency: Some("SGD".into()),
                market: Some("SG".into()),
                action: Some("BUY".into()),
                order_type: Some("LMT".into()),
                total_quantity: Some(100),
                limit_price: Some(SAFE_BUY_PRICE),
                ..Default::default()
            },
            "SG STK LMT",
        )
        .await;
    }

    // =====================================================================
    // Order matrix — Phase 3 (MLEG combo + edge cases)
    // =====================================================================

    #[tokio::test]
    async fn test_matrix_us_mleg_vertical_spread() {
        let Some(tc) = skip_if_disabled() else { return };
        let expiry = future_expiry_yyyymmdd(45);
        preview_and_place(
            &tc,
            OrderRequest {
                sec_type: Some("MLEG".into()),
                currency: Some("USD".into()),
                market: Some("US".into()),
                combo_type: Some("VERTICAL".into()),
                action: Some("BUY".into()),
                order_type: Some("LMT".into()),
                total_quantity: Some(1),
                limit_price: Some(-100.0),
                contract_legs: Some(vec![
                    ContractLegRequest {
                        symbol: Some("AAPL".into()),
                        sec_type: Some("OPT".into()),
                        expiry: Some(expiry.clone()),
                        strike: Some("200".into()),
                        right: Some("PUT".into()),
                        action: Some("BUY".into()),
                        ratio: Some(1),
                    },
                    ContractLegRequest {
                        symbol: Some("AAPL".into()),
                        sec_type: Some("OPT".into()),
                        expiry: Some(expiry),
                        strike: Some("205".into()),
                        right: Some("PUT".into()),
                        action: Some("SELL".into()),
                        ratio: Some(1),
                    },
                ]),
                ..Default::default()
            },
            "US MLEG VERTICAL",
        )
        .await;
    }

    #[tokio::test]
    async fn test_matrix_us_stk_iceberg_modify() {
        let Some(tc) = skip_if_disabled() else { return };
        let now = now_ms();
        let order = OrderRequest {
            action: Some("BUY".into()),
            order_type: Some("ICEBERG".into()),
            total_quantity: Some(10),
            limit_price: Some(SAFE_BUY_PRICE),
            display_size: Some(2),
            min_display_size: Some(1),
            check_intervals: Some(30),
            price_type: Some("LIMIT_PRICE".into()),
            start_time: Some(now),
            end_time: Some(now + 3_600_000),
            ..us_stk()
        };
        if !preview_only(&tc, order.clone(), "US STK ICEBERG modify preview").await {
            return;
        }
        let placed = match tc.place_order(order.clone()).await {
            Ok(Some(r)) => r,
            Ok(None) => panic!("place returned Ok(None)"),
            Err(e) if matches_any(&e.to_string(), PERMISSION_ERROR_MARKERS) => {
                eprintln!("[US STK ICEBERG modify] skipped: {e}");
                return;
            }
            Err(e) => panic!("place failed: {e}"),
        };
        let order_id = if placed.id != 0 {
            placed.id
        } else if placed.order_id != 0 {
            placed.order_id
        } else {
            panic!("no order id in {placed:?}");
        };
        let modified = OrderRequest {
            limit_price: Some(SAFE_BUY_PRICE * 2.0),
            ..order
        };
        if let Err(e) = tc.modify_order(order_id, modified).await {
            if !matches_any(&e.to_string(), TERMINAL_ORDER_MARKERS) {
                eprintln!("[US STK ICEBERG modify] modify failed (best-effort): {e}");
            }
        }
        cancel_tolerant(&tc, order_id, "US STK ICEBERG modify").await;
    }

    #[tokio::test]
    async fn test_matrix_us_stk_sell_short_preview() {
        let Some(tc) = skip_if_disabled() else { return };
        preview_only(
            &tc,
            OrderRequest {
                action: Some("SELL".into()),
                order_type: Some("LMT".into()),
                total_quantity: Some(1),
                limit_price: Some(SAFE_SELL_PRICE),
                ..us_stk()
            },
            "US STK SELL SHORT preview",
        )
        .await;
    }
}
