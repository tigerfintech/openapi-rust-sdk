//! Integration tests for QuoteClient — require real API credentials.
//! Run with: TIGER_RUN_INTEG=true cargo test --test integ_quote -- --nocapture

mod integ_support;

#[cfg(test)]
mod tests {
    use super::integ_support;
    use tigeropen::model::quote_requests::{
        BriefRequest, KlineRequest, OptionChainItem, OptionChainRequest,
    };
    use tigeropen::model::quote::{
        Brief, CapitalFlow, CorporateAction, CorporateActionRequest, Kline, KlineItem,
        OptionChain, OptionExpiration, OptionLeg, QuotePermission, Timeline, TimelineItem,
    };
    use tigeropen::quote::QuoteClient;

    #[tokio::test]
    async fn test_integ_get_market_state() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let result = client.get_market_state("US").await;
        assert!(
            result.is_ok(),
            "get_market_state should succeed: {:?}",
            result
        );
        let data = result.unwrap();
        assert!(!data.is_empty(), "market state list should not be empty");
        let ms = &data[0];
        assert!(
            !ms.market.is_empty(),
            "MarketState.market should be non-empty, got {:?}",
            ms.market
        );
        assert!(
            !(ms.market_status.is_empty() && ms.status.is_empty()),
            "MarketState should have a status (marketStatus={:?}, status={:?})",
            ms.market_status,
            ms.status
        );
    }

    #[tokio::test]
    async fn test_integ_get_real_time_quote() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = BriefRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        };
        let result = client.get_real_time_quote(req).await;
        assert!(
            result.is_ok(),
            "get_real_time_quote should succeed: {:?}",
            result
        );
        let data: Vec<Brief> = result.unwrap();
        assert!(
            !data.is_empty(),
            "brief result should not be empty for AAPL"
        );
        let b = &data[0];
        assert_eq!(
            b.symbol, "AAPL",
            "Brief.symbol should be AAPL, got {:?}",
            b.symbol
        );
        assert!(
            b.latest_price > 0.0,
            "Brief.latest_price should be > 0, got {}",
            b.latest_price
        );
        assert!(
            b.latest_time > 0,
            "Brief.latest_time should be a non-zero timestamp, got {}",
            b.latest_time
        );
        assert!(
            b.high >= b.low,
            "Brief.high ({}) should be >= low ({})",
            b.high,
            b.low
        );
    }

    #[tokio::test]
    async fn test_integ_get_kline() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = KlineRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            period: Some("day".to_string()),
            limit: Some(5),
            ..Default::default()
        };
        let result = client.get_kline(req).await;
        assert!(result.is_ok(), "get_kline should succeed: {:?}", result);
        let data: Vec<Kline> = result.unwrap();
        assert!(!data.is_empty(), "kline result should not be empty");
        let k = &data[0];
        assert_eq!(
            k.symbol, "AAPL",
            "Kline.symbol should be AAPL, got {:?}",
            k.symbol
        );
        assert!(
            !k.items.is_empty(),
            "Kline.items should not be empty for AAPL"
        );
        let item: &KlineItem = &k.items[0];
        assert!(
            item.time > 0,
            "KlineItem.time should be non-zero, got {}",
            item.time
        );
        assert!(
            item.close > 0.0,
            "KlineItem.close should be > 0, got {}",
            item.close
        );
        assert!(
            item.high >= item.low,
            "KlineItem.high ({}) should be >= low ({})",
            item.high,
            item.low
        );
    }

    #[tokio::test]
    async fn test_integ_get_timeline() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let result = client.get_timeline(&["AAPL"]).await;
        assert!(result.is_ok(), "get_timeline should succeed: {:?}", result);
        let data: Vec<Timeline> = result.unwrap();
        assert!(
            !data.is_empty(),
            "timeline result should not be empty for AAPL"
        );
        let t = &data[0];
        assert_eq!(
            t.symbol, "AAPL",
            "Timeline.symbol should be AAPL, got {:?}",
            t.symbol
        );
        // Timeline buckets may legitimately be empty outside trading hours.
        // When items exist, validate their fields.
        for bucket in [t.intraday.as_ref(), t.pre_hours.as_ref(), t.after_hours.as_ref()]
            .iter()
            .copied()
            .flatten()
        {
            if bucket.items.is_empty() {
                continue;
            }
            let item: &TimelineItem = &bucket.items[0];
            assert!(
                item.time > 0,
                "TimelineItem.time should be non-zero, got {}",
                item.time
            );
            assert!(
                item.price > 0.0,
                "TimelineItem.price should be > 0, got {}",
                item.price
            );
            break; // only check first non-empty bucket
        }
    }

    #[tokio::test]
    async fn test_integ_get_option_expiration() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let result = client.get_option_expiration(&["AAPL"], None).await;
        assert!(
            result.is_ok(),
            "get_option_expiration should succeed: {:?}",
            result
        );
        let data: Vec<OptionExpiration> = result.unwrap();
        assert!(
            !data.is_empty(),
            "option expiration result should not be empty"
        );
        let exp = &data[0];
        assert_eq!(
            exp.symbol, "AAPL",
            "OptionExpiration.symbol should be AAPL, got {:?}",
            exp.symbol
        );
        assert!(
            !exp.dates.is_empty(),
            "OptionExpiration.dates should not be empty for AAPL"
        );
        assert!(
            !exp.timestamps.is_empty(),
            "OptionExpiration.timestamps should not be empty (should match dates)"
        );
        assert!(
            !exp.option_symbols.is_empty(),
            "OptionExpiration.option_symbols should not be empty"
        );
    }

    #[tokio::test]
    async fn test_integ_get_option_chain() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);

        // First fetch expirations; skip if none available.
        let exps = client.get_option_expiration(&["AAPL"], None).await;
        let exps = match exps {
            Ok(e) if !e.is_empty() && !e[0].dates.is_empty() => e,
            _ => return, // no expiry available — skip
        };
        let mid = exps[0].dates.len() / 2;
        let expiry_str = exps[0].dates[mid].clone();

        let item = match OptionChainItem::from_date("AAPL", &expiry_str) {
            Ok(it) => it,
            Err(_) => return, // date parse failed — skip
        };
        let req = OptionChainRequest::new(vec![item]);
        let result = client.get_option_chain(req).await;
        assert!(result.is_ok(), "get_option_chain should succeed: {:?}", result);
        let data: Vec<OptionChain> = result.unwrap();
        assert!(!data.is_empty(), "option chain result should not be empty");
        let chain = &data[0];
        assert_eq!(
            chain.symbol, "AAPL",
            "OptionChain.symbol should be AAPL, got {:?}",
            chain.symbol
        );
        assert!(
            chain.expiry > 0,
            "OptionChain.expiry should be non-zero, got {}",
            chain.expiry
        );
        assert!(
            !chain.items.is_empty(),
            "OptionChain.items should not be empty"
        );
        let row = &chain.items[0];
        assert!(
            row.call.is_some() || row.put.is_some(),
            "OptionChainRow should have at least a call or put leg"
        );
        if let Some(call) = &row.call {
            let leg: &OptionLeg = call;
            assert!(
                !leg.identifier.is_empty(),
                "Call leg identifier should be non-empty"
            );
            assert!(
                !leg.strike.is_empty(),
                "Call leg strike should be non-empty"
            );
        }
        if let Some(put) = &row.put {
            let leg: &OptionLeg = put;
            assert!(
                !leg.identifier.is_empty(),
                "Put leg identifier should be non-empty"
            );
            assert!(
                !leg.strike.is_empty(),
                "Put leg strike should be non-empty"
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_corporate_action() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = CorporateActionRequest {
            symbols: vec!["AAPL".to_string()],
            market: "US".to_string(),
            action_type: "DIVIDEND".to_string(),
            begin_date: "2024-01-01".to_string(),
            end_date: "2025-12-31".to_string(),
        };
        let result = client.get_corporate_action(req).await;
        assert!(
            result.is_ok(),
            "get_corporate_action should succeed: {:?}",
            result
        );
        let data: Vec<CorporateAction> = result.unwrap();
        assert!(
            !data.is_empty(),
            "corporate action result should not be empty for AAPL DIVIDEND over 2-year range"
        );
        let ca = &data[0];
        assert_eq!(
            ca.symbol, "AAPL",
            "CorporateAction.symbol should be AAPL, got {:?}",
            ca.symbol
        );
        assert!(
            !ca.action_type.is_empty(),
            "CorporateAction.action_type should be non-empty"
        );
        assert!(
            !ca.execute_date.is_empty(),
            "CorporateAction.execute_date should be a non-empty date string"
        );
    }

    #[tokio::test]
    async fn test_integ_get_capital_flow() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let result = client.get_capital_flow("AAPL", "US", "1d").await;
        assert!(
            result.is_ok(),
            "get_capital_flow should succeed: {:?}",
            result
        );
        let data: Option<CapitalFlow> = result.unwrap();
        let data = data.expect("capital flow result should not be None for AAPL daily");
        assert_eq!(
            data.symbol, "AAPL",
            "CapitalFlow.symbol should be AAPL, got {:?}",
            data.symbol
        );
        if !data.items.is_empty() {
            let item = &data.items[0];
            assert!(
                item.timestamp > 0,
                "CapitalFlowItem.timestamp should be non-zero, got {}",
                item.timestamp
            );
        }
    }

    #[tokio::test]
    async fn test_integ_grab_quote_permission() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let result = client.grab_quote_permission().await;
        assert!(
            result.is_ok(),
            "grab_quote_permission should succeed: {:?}",
            result
        );
        let data: Vec<QuotePermission> = result.unwrap();
        // Permission list may be empty if the account has no active entitlements.
        // When entries exist, validate identifier fields.
        for (i, p) in data.iter().enumerate() {
            assert!(
                !p.name.is_empty(),
                "QuotePermission[{}].name should be non-empty",
                i
            );
        }
    }
}
