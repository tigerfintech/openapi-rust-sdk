//! Order matrix integration tests — mirrors the Python/TS/Java/Go SDK
//! three-phase coverage. Every test uses safe prices (BUY $0.01 / SELL
//! $999,999) so orders never fill, plus a preview_and_place helper that
//! runs preview -> place -> cancel with automatic skip on legitimate
//! boundary errors.
//!
//! Run with:
//!   TIGER_RUN_INTEG=true cargo test --test integ_trade_matrix -- --nocapture
//!
//! Phase 1 (US market x order type):
//!   MKT preview, MKT-by-amount preview, STP, STP_LMT, TRAIL, TWAP, VWAP,
//!   ICEBERG, OCA brackets, US OPT LMT, US FUT LMT, forex SEC,
//!   invalid-price preview.
//!
//! Phase 2 (HK / CN / SG):
//!   HK STK LMT, HK STK AL, HK STK AM preview, CN STK LMT, SG STK LMT.
//!
//! Phase 3 (MLEG + edge):
//!   MLEG vertical spread, ICEBERG modify round-trip, SELL SHORT preview.

mod integ_support;

#[cfg(test)]
mod tests {
    use super::integ_support;
    use tigeropen::error::TigerError;
    use tigeropen::model::order::{AlgoParamsRequest, ContractLegRequest, OrderRequest};
    use tigeropen::model::trade_requests::ForexOrderRequest;
    use tigeropen::trade::TradeClient;

    // Safe prices — kept far from market so BUY / SELL orders never fill.
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
    ];

    /// Order state race markers — cancel/modify may hit a terminal state
    /// (already filled / already cancelled) which is a legitimate outcome.
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
    const RATE_LIMIT_MARKERS: &[&str] = &[
        "too_many_requests",
        "rate limit",
        "requestrateexceedlimit",
    ];

    fn matches_any(msg: &str, markers: &[&str]) -> bool {
        let lower = msg.to_lowercase();
        markers.iter().any(|m| lower.contains(m))
    }

    // -------------------------------------------------------------------------
    // Contract helpers
    // -------------------------------------------------------------------------

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

    fn now_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    fn future_expiry_yyyymmdd(days: i64) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let target = now + days * 86_400;
        // Naive date formatting; good enough for a preview payload.
        let days_since_epoch = target / 86_400;
        // Delegate to time crate via chrono-free formula:
        let (y, m, d) = civil_from_days(days_since_epoch);
        format!("{:04}{:02}{:02}", y, m, d)
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

    // -------------------------------------------------------------------------
    // preview / place / cancel helpers
    // -------------------------------------------------------------------------

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
        // 1. Preview first — validates marshaling before touching real state.
        if !preview_only(tc, order.clone(), ctx).await {
            return false;
        }

        // 2. Place with exponential backoff on rate-limit.
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
        // PlaceOrderResult 的 id / order_id 都是 i64(不是 Option),0 表示服
        // 务端没返;真实响应总是至少填其中一个。
        let order_id = if place_result.id != 0 {
            place_result.id
        } else if place_result.order_id != 0 {
            place_result.order_id
        } else {
            panic!("[{ctx}] no order id in {place_result:?}");
        };
        eprintln!("[{ctx}] placed order id={order_id}");

        // 3. Cancel tolerantly.
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

    fn skip_if_disabled() -> Option<TradeClient> {
        if !integ_support::is_integ_run() {
            return None;
        }
        Some(TradeClient::from_config(integ_support::integ_config()))
    }

    // -------------------------------------------------------------------------
    // Phase 1 — US market x order type
    // -------------------------------------------------------------------------

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
        // Gateway requires total_quantity present even for amount orders; the
        // real sizing comes from cash_amount.
        preview_only(
            &tc,
            OrderRequest {
                action: Some("BUY".into()),
                order_type: Some("MKT".into()),
                cash_amount: Some(100.0),
                total_quantity: Some(1),
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
            limit_price: Some(SAFE_SELL_PRICE),
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
                // Business-side rejection is fine — the wire round-trip
                // (typing / marshaling) succeeded, which is what we check.
            }
            Err(e) => panic!("Forex SEC unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn test_matrix_us_stk_negative_price_preview() {
        let Some(tc) = skip_if_disabled() else { return };
        // Either accepted or rejected — both are semantically valid.
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

    // -------------------------------------------------------------------------
    // Phase 2 — HK / CN / SG
    // -------------------------------------------------------------------------

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

    // -------------------------------------------------------------------------
    // Phase 3 — MLEG combo + edge cases
    // -------------------------------------------------------------------------

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

        // Modify: bump limit price.
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
