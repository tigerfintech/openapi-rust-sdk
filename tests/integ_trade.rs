//! Integration tests for TradeClient — require real API credentials.
//! Only read-only operations (query orders, positions, assets) — no order placement.
//! Run with: TIGER_RUN_INTEG=true cargo test --test integ_trade -- --nocapture

mod integ_support;

#[cfg(test)]
mod tests {
    use super::integ_support;
    use tigeropen::model::trade_requests::{
        AggregateAssetsRequest, AnalyticsAssetRequest, AssetsRequest, DerivativeContractsRequest,
        EstimateTradableQuantityRequest, FundDetailsRequest, FundingHistoryRequest,
        GetOrderRequest, ManagedAccountsRequest, OptionExercisePositionRequest,
        OptionExerciseRecordsRequest, OrderTransactionsRequest, OrdersRequest,
        PositionTransferExternalRecordsRequest, PositionTransferRecordsRequest, PositionsRequest,
        SegmentFundRequest,
    };
    use tigeropen::model::{
        AggregateAssets, AnalyticsAsset, Asset, Contract, EstimateTradableQuantity, FundDetails,
        FundingHistoryItem, ManagedAccount, Order, Position, PrimeAsset, SegmentFundAvailableItem,
        SegmentFundHistoryItem, Transaction,
    };
    use tigeropen::trade::TradeClient;

    #[tokio::test]
    async fn test_integ_get_positions() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = PositionsRequest::default();
        let result = client.get_positions(req).await;
        assert!(
            result.is_ok(),
            "get_positions should succeed: {:?}",
            result
        );
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
        assert!(!data.is_empty(), "contract result should not be empty for AAPL");
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
        assert!(
            result.is_ok(),
            "get_contracts should succeed: {:?}",
            result
        );
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
        // quote_contract is designed for derivatives (OPT/WAR/IOPT) with expiry.
        // For STK, expiry is ignored; query AAPL as a basic contract.
        let result = client.get_quote_contract("AAPL", "STK", "").await;
        assert!(
            result.is_ok(),
            "get_quote_contract should succeed: {:?}",
            result
        );
        let data: Vec<Contract> = result.unwrap();
        assert!(!data.is_empty(), "quote_contract result should not be empty");
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
        let req = OrdersRequest::default();
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
        let req = GetOrderRequest {
            order_id: Some(order_id),
            ..Default::default()
        };
        let result = client.get_order(req).await;
        assert!(result.is_ok(), "get_order should succeed: {:?}", result);
        if let Some(o) = result.unwrap() {
            assert!(o.order_id == order_id || o.id > 0);
        }
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
        let req = DerivativeContractsRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            sec_type: Some("OPT".to_string()),
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
        assert!(
            result.is_ok(),
            "get_aggregate_assets should succeed: {:?}",
            result
        );
        let data: Option<AggregateAssets> = result.unwrap();
        if let Some(a) = data {
            assert!(
                a.net_liquidation >= 0.0,
                "AggregateAssets.net_liquidation should be >= 0, got {}",
                a.net_liquidation
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_estimate_tradable_quantity() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = EstimateTradableQuantityRequest {
            symbol: Some("AAPL".to_string()),
            sec_type: Some("STK".to_string()),
            action: Some("BUY".to_string()),
            order_type: Some("MARKET".to_string()),
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
        let req = SegmentFundRequest::default();
        let result = client.get_segment_fund_available(req).await;
        assert!(
            result.is_ok(),
            "get_segment_fund_available should succeed: {:?}",
            result
        );
        let _data: Vec<SegmentFundAvailableItem> = result.unwrap();
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
        let req = FundDetailsRequest {
            limit: Some(5),
            ..Default::default()
        };
        let result = client.get_fund_details(req).await;
        assert!(
            result.is_ok(),
            "get_fund_details should succeed: {:?}",
            result
        );
        let data: Vec<FundDetails> = result.unwrap();
        if !data.is_empty() {
            assert!(
                !data[0].account.is_empty(),
                "FundDetails.account should be non-empty"
            );
        }
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
        let req = PositionTransferRecordsRequest {
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
        let req = PositionTransferExternalRecordsRequest {
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
        assert!(result.is_ok(), "query_token should succeed: {:?}", result);
        let token = result.unwrap();
        assert!(
            !token.is_empty(),
            "refreshed token string should be non-empty"
        );
    }
}
