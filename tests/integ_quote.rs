//! Integration tests for QuoteClient — require real API credentials.
//! Run with: TIGER_RUN_INTEG=true cargo test --test integ_quote -- --nocapture

mod integ_support;

#[cfg(test)]
mod tests {
    use super::integ_support;
    use tigeropen::model::quote_requests::{
        AllFutureContractsRequest, BriefRequest, DelayedQuoteRequest, FinancialCurrencyRequest,
        FinancialExchangeRateRequest, FundContractsRequest, FundHistoryQuoteRequest,
        FundQuoteRequest, FundSymbolsRequest, FutureContinuousContractsRequest,
        FutureContractSingleRequest, FutureDepthRequest, FutureHistoryMainContractRequest,
        FutureKlineByPageRequest, FutureKlineRequest, FutureRealTimeQuoteRequest,
        FutureTradeTicksRequest, FutureTradingTimesRequest, IndustryListRequest,
        IndustryStocksRequest, KlineByPageRequest, KlineQuotaRequest, KlineRequest,
        MarketScannerTagsRequest, OptionAnalysisRequest, OptionChainItem, OptionChainRequest,
        OptionContractItem, OptionDepthRequest, OptionKlineItem, OptionKlineRequest,
        OptionQuoteRequest, OptionQueryItem, OptionSymbolsRequest, OptionTimelineRequest,
        OptionTradeTicksRequest, QuoteDepthRequest, QuoteOvernightRequest, QuotePermissionRequest,
        ShortInterestRequest, StockBrokerRequest, StockDetailsRequest, StockFundamentalRequest,
        StockIndustryRequest, SymbolsRequest, TimelineHistoryRequest, TradeMetasRequest,
        TradeRankRequest, TradeTickRequest, TradingCalendarRequest, WarrantFilterRequest,
        WarrantQuoteRequest,
    };
    use tigeropen::model::quote::{
        Brief, CapitalDistribution, CapitalFlow, CorporateAction, CorporateActionRequest, Depth,
        DepthLevel, ExchangeRate, FinancialCurrency, FundContractInfo, FundHistoryQuote,
        FundQuote, FutureContractInfo, FutureDepth, FutureExchange, FutureKline, FutureKlineItem,
        FutureMainContractHistory, FutureQuote, FutureTradeTickItem, FutureTradingTime,
        IndustryItem, IndustryStock, Kline, KlineItem, KlineQuota, MarketScannerTagGroup,
        OptionAnalysis, OptionChain, OptionExpiration, OptionLeg, OptionSymbol,
        QuoteOvernight, QuotePermission, ShortInterest, StockBroker, StockDetail, StockIndustry,
        SymbolName, Timeline, TimelineItem, TradeMeta, TradeRankItem, TradeTick, TradeTickItem,
        TradingCalendarItem, WarrantBrief,
    };
    use tigeropen::model::quote as qm;
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
        // If no expiry dates are returned (e.g. market closed, symbol has no
        // options, or entitlements missing), the test cannot exercise the chain
        // endpoint. Return early as a skip — not a pass or fail — since there
        // is no data to assert against.
        let exps = match exps {
            Ok(e) if !e.is_empty() && !e[0].dates.is_empty() => e,
            _ => return,
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
        // Capital flow may be None outside trading hours; skip when empty.
        let data = match data {
            Some(d) => d,
            None => return, // non-trading hours, data may be empty
        };
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

    // ── 基础行情扩展 ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_integ_get_trade_tick() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = TradeTickRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            limit: Some(5),
            ..Default::default()
        };
        let result = client.get_trade_tick(req).await;
        assert!(result.is_ok(), "get_trade_tick should succeed: {:?}", result);
        let data: Vec<TradeTick> = result.unwrap();
        assert!(
            !data.is_empty(),
            "trade_tick result should not be empty for AAPL"
        );
        let t = &data[0];
        assert_eq!(
            t.symbol, "AAPL",
            "TradeTick.symbol should be AAPL, got {:?}",
            t.symbol
        );
        if !t.items.is_empty() {
            let item: &TradeTickItem = &t.items[0];
            assert!(
                item.time > 0,
                "TradeTickItem.time should be non-zero, got {}",
                item.time
            );
            assert!(
                item.price > 0.0,
                "TradeTickItem.price should be > 0, got {}",
                item.price
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_quote_depth() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = QuoteDepthRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            market: Some("US".to_string()),
            ..Default::default()
        };
        let result = client.get_quote_depth(req).await;
        assert!(result.is_ok(), "get_quote_depth should succeed: {:?}", result);
        let data: Vec<Depth> = result.unwrap();
        assert!(
            !data.is_empty(),
            "quote_depth result should not be empty for AAPL"
        );
        let d = &data[0];
        assert_eq!(
            d.symbol, "AAPL",
            "Depth.symbol should be AAPL, got {:?}",
            d.symbol
        );
        // Depth may be empty outside trading hours; when levels exist, validate fields.
        for level in d.asks.iter().take(1) {
            let _: &DepthLevel = level;
            assert!(
                level.price >= 0.0,
                "DepthLevel.price should be >= 0, got {}",
                level.price
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_symbols() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = SymbolsRequest {
            market: Some("US".to_string()),
            sec_type: Some("STK".to_string()),
            ..Default::default()
        };
        let result = client.get_symbols(req).await;
        assert!(result.is_ok(), "get_symbols should succeed: {:?}", result);
        let data: Vec<String> = result.unwrap();
        assert!(!data.is_empty(), "all_symbols list should not be empty");
        assert!(
            data.iter().any(|s| !s.is_empty()),
            "symbol strings should be non-empty"
        );
    }

    #[tokio::test]
    async fn test_integ_get_symbol_names() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = SymbolsRequest {
            market: Some("US".to_string()),
            sec_type: Some("STK".to_string()),
            ..Default::default()
        };
        let result = client.get_symbol_names(req).await;
        assert!(
            result.is_ok(),
            "get_symbol_names should succeed: {:?}",
            result
        );
        let data: Vec<SymbolName> = result.unwrap();
        assert!(!data.is_empty(), "all_symbol_names list should not be empty");
        let sn = &data[0];
        assert!(
            !sn.symbol.is_empty(),
            "SymbolName.symbol should be non-empty, got {:?}",
            sn.symbol
        );
    }

    #[tokio::test]
    async fn test_integ_get_trade_metas() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = TradeMetasRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        };
        let result = client.get_trade_metas(req).await;
        assert!(result.is_ok(), "get_trade_metas should succeed: {:?}", result);
        let data: Vec<TradeMeta> = result.unwrap();
        assert!(
            !data.is_empty(),
            "trade_metas result should not be empty for AAPL"
        );
        let m = &data[0];
        assert_eq!(
            m.symbol, "AAPL",
            "TradeMeta.symbol should be AAPL, got {:?}",
            m.symbol
        );
        assert!(
            m.lot_size > 0,
            "TradeMeta.lot_size should be > 0, got {}",
            m.lot_size
        );
    }

    #[tokio::test]
    async fn test_integ_get_stock_details() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = StockDetailsRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        };
        let result = client.get_stock_details(req).await;
        assert!(
            result.is_ok(),
            "get_stock_details should succeed: {:?}",
            result
        );
        let data: Vec<StockDetail> = result.unwrap();
        assert!(
            !data.is_empty(),
            "stock_detail result should not be empty for AAPL"
        );
        let s = &data[0];
        assert_eq!(
            s.symbol, "AAPL",
            "StockDetail.symbol should be AAPL, got {:?}",
            s.symbol
        );
        assert!(
            !s.exchange.is_empty(),
            "StockDetail.exchange should be non-empty"
        );
    }

    #[tokio::test]
    async fn test_integ_get_delayed_quote() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = DelayedQuoteRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        };
        let result = client.get_delayed_quote(req).await;
        assert!(
            result.is_ok(),
            "get_delayed_quote should succeed: {:?}",
            result
        );
        let data: Vec<Brief> = result.unwrap();
        assert!(
            !data.is_empty(),
            "delayed quote result should not be empty for AAPL"
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
    }

    #[tokio::test]
    async fn test_integ_get_kline_by_page() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = KlineByPageRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            period: Some("day".to_string()),
            total_size: Some(5),
            page_size: Some(5),
            ..Default::default()
        };
        let result = client.get_kline_by_page(req).await;
        assert!(
            result.is_ok(),
            "get_kline_by_page should succeed: {:?}",
            result
        );
        let data: Vec<KlineItem> = result.unwrap();
        assert!(!data.is_empty(), "kline_by_page items should not be empty");
        let item: &KlineItem = &data[0];
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
    }

    #[tokio::test]
    async fn test_integ_get_timeline_history() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = TimelineHistoryRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            date: Some("2024-01-02".to_string()),
            ..Default::default()
        };
        let result = client.get_timeline_history(req).await;
        assert!(
            result.is_ok(),
            "get_timeline_history should succeed: {:?}",
            result
        );
        let data: Vec<Timeline> = result.unwrap();
        assert!(
            !data.is_empty(),
            "history_timeline result should not be empty for AAPL"
        );
        let t = &data[0];
        assert_eq!(
            t.symbol, "AAPL",
            "Timeline.symbol should be AAPL, got {:?}",
            t.symbol
        );
    }

    #[tokio::test]
    async fn test_integ_get_trade_rank() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = TradeRankRequest {
            market: Some("US".to_string()),
            ..Default::default()
        };
        let result = client.get_trade_rank(req).await;
        assert!(result.is_ok(), "get_trade_rank should succeed: {:?}", result);
        let data: Vec<TradeRankItem> = result.unwrap();
        assert!(!data.is_empty(), "trade_rank result should not be empty");
        let r = &data[0];
        assert!(
            !r.symbol.is_empty(),
            "TradeRankItem.symbol should be non-empty, got {:?}",
            r.symbol
        );
        assert!(
            r.latest_price > 0.0,
            "TradeRankItem.latest_price should be > 0, got {}",
            r.latest_price
        );
    }

    #[tokio::test]
    async fn test_integ_get_short_interest() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = ShortInterestRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        };
        let result = client.get_short_interest(req).await;
        assert!(
            result.is_ok(),
            "get_short_interest should succeed: {:?}",
            result
        );
        let data: Vec<ShortInterest> = result.unwrap();
        assert!(
            !data.is_empty(),
            "short_interest result should not be empty for AAPL"
        );
        let s = &data[0];
        assert_eq!(
            s.symbol, "AAPL",
            "ShortInterest.symbol should be AAPL, got {:?}",
            s.symbol
        );
    }

    #[tokio::test]
    async fn test_integ_get_stock_broker() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = StockBrokerRequest {
            symbol: Some("AAPL".to_string()),
            ..Default::default()
        };
        let result = client.get_stock_broker(req).await;
        assert!(
            result.is_ok(),
            "get_stock_broker should succeed: {:?}",
            result
        );
        let data: Option<StockBroker> = result.unwrap();
        if let Some(b) = data {
            assert_eq!(
                b.symbol, "AAPL",
                "StockBroker.symbol should be AAPL, got {:?}",
                b.symbol
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_stock_fundamental() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = StockFundamentalRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            market: Some("US".to_string()),
            ..Default::default()
        };
        let result = client.get_stock_fundamental(req).await;
        assert!(
            result.is_ok(),
            "get_stock_fundamental should succeed: {:?}",
            result
        );
        let data = result.unwrap();
        assert!(
            !data.is_empty(),
            "stock_fundamental map should not be empty"
        );
    }

    #[tokio::test]
    async fn test_integ_get_stock_industry() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = StockIndustryRequest {
            symbol: Some("AAPL".to_string()),
            market: Some("US".to_string()),
            ..Default::default()
        };
        let result = client.get_stock_industry(req).await;
        assert!(
            result.is_ok(),
            "get_stock_industry should succeed: {:?}",
            result
        );
        let data: Vec<StockIndustry> = result.unwrap();
        assert!(
            !data.is_empty(),
            "stock_industry result should not be empty for AAPL"
        );
        let s = &data[0];
        assert_eq!(
            s.symbol, "AAPL",
            "StockIndustry.symbol should be AAPL, got {:?}",
            s.symbol
        );
    }

    #[tokio::test]
    async fn test_integ_get_quote_permission() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = QuotePermissionRequest::default();
        let result = client.get_quote_permission(req).await;
        assert!(
            result.is_ok(),
            "get_quote_permission should succeed: {:?}",
            result
        );
        let data: Vec<QuotePermission> = result.unwrap();
        for (i, p) in data.iter().enumerate() {
            assert!(
                !p.name.is_empty(),
                "QuotePermission[{}].name should be non-empty",
                i
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_kline_quota() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = KlineQuotaRequest::default();
        let result = client.get_kline_quota(req).await;
        assert!(result.is_ok(), "get_kline_quota should succeed: {:?}", result);
        let data: Vec<KlineQuota> = result.unwrap();
        // Quota may legitimately be empty; when entries exist, validate method field.
        if !data.is_empty() {
            let q = &data[0];
            assert!(
                !q.method.is_empty(),
                "KlineQuota.method should be non-empty, got {:?}",
                q.method
            );
        }
    }

    // ── 期权扩展 ──────────────────────────────────────────────────────────

    /// Helper: fetch a real option leg (call preferred) + chain expiry for a symbol.
    async fn first_option_leg(
        client: &QuoteClient,
        symbol: &str,
    ) -> Option<(i64, OptionLeg)> {
        let exps = client.get_option_expiration(&[symbol], None).await.ok()?;
        let exp = exps.first()?;
        if exp.dates.is_empty() {
            return None;
        }
        let mid = exp.dates.len() / 2;
        let item = OptionChainItem::from_date(symbol, &exp.dates[mid]).ok()?;
        let req = OptionChainRequest::new(vec![item]);
        let chains = client.get_option_chain(req).await.ok()?;
        let chain = chains.first()?;
        for row in &chain.items {
            if let Some(call) = &row.call {
                return Some((chain.expiry, call.clone()));
            }
            if let Some(put) = &row.put {
                return Some((chain.expiry, put.clone()));
            }
        }
        None
    }

    #[tokio::test]
    async fn test_integ_get_option_quote() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let (expiry, leg) = match first_option_leg(&client, "AAPL").await {
            Some(v) => v,
            None => return,
        };
        let item = OptionContractItem::new("AAPL", expiry, &leg.right, &leg.strike);
        let req = OptionQuoteRequest::new(vec![item]);
        let result = client.get_option_quote(req).await;
        assert!(result.is_ok(), "get_option_quote should succeed: {:?}", result);
        let data: Vec<Brief> = result.unwrap();
        assert!(
            !data.is_empty(),
            "option_quote result should not be empty"
        );
        let b = &data[0];
        assert!(
            b.latest_price >= 0.0,
            "Brief.latest_price should be >= 0, got {}",
            b.latest_price
        );
    }

    #[tokio::test]
    async fn test_integ_get_option_kline() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let (expiry, leg) = match first_option_leg(&client, "AAPL").await {
            Some(v) => v,
            None => return,
        };
        let item =
            OptionKlineItem::new("AAPL", expiry, &leg.right, &leg.strike, "day");
        let req = OptionKlineRequest {
            option_query: Some(vec![item]),
            ..Default::default()
        };
        let result = client.get_option_kline(req).await;
        assert!(result.is_ok(), "get_option_kline should succeed: {:?}", result);
        let data: Vec<Kline> = result.unwrap();
        assert!(!data.is_empty(), "option_kline result should not be empty");
        if !data[0].items.is_empty() {
            let item: &KlineItem = &data[0].items[0];
            assert!(
                item.time > 0,
                "KlineItem.time should be non-zero, got {}",
                item.time
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_option_trade_ticks() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let (expiry, leg) = match first_option_leg(&client, "AAPL").await {
            Some(v) => v,
            None => return,
        };
        let query = OptionQueryItem {
            symbol: Some("AAPL".to_string()),
            expiry: Some(expiry),
            strike: Some(leg.strike.clone()),
            right: Some(leg.right.clone()),
            limit: Some(5),
            ..Default::default()
        };
        let req = OptionTradeTicksRequest {
            contracts: Some(vec![query]),
            ..Default::default()
        };
        let result = client.get_option_trade_ticks(req).await;
        assert!(
            result.is_ok(),
            "get_option_trade_ticks should succeed: {:?}",
            result
        );
        let data: Vec<TradeTick> = result.unwrap();
        if !data.is_empty() && !data[0].items.is_empty() {
            let item: &TradeTickItem = &data[0].items[0];
            assert!(
                item.time > 0,
                "TradeTickItem.time should be non-zero, got {}",
                item.time
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_option_timeline() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let (expiry, leg) = match first_option_leg(&client, "AAPL").await {
            Some(v) => v,
            None => return,
        };
        let query = OptionQueryItem {
            symbol: Some("AAPL".to_string()),
            expiry: Some(expiry),
            strike: Some(leg.strike.clone()),
            right: Some(leg.right.clone()),
            ..Default::default()
        };
        let req = OptionTimelineRequest {
            option_query: Some(vec![query]),
            ..Default::default()
        };
        let result = client.get_option_timeline(req).await;
        assert!(
            result.is_ok(),
            "get_option_timeline should succeed: {:?}",
            result
        );
        let data: Vec<Timeline> = result.unwrap();
        if !data.is_empty() {
            assert_eq!(
                data[0].symbol, "AAPL",
                "Timeline.symbol should be AAPL, got {:?}",
                data[0].symbol
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_option_depth() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let (expiry, leg) = match first_option_leg(&client, "AAPL").await {
            Some(v) => v,
            None => return,
        };
        let query = OptionQueryItem {
            symbol: Some("AAPL".to_string()),
            expiry: Some(expiry),
            strike: Some(leg.strike.clone()),
            right: Some(leg.right.clone()),
            ..Default::default()
        };
        let req = OptionDepthRequest {
            option_basic: Some(vec![query]),
            ..Default::default()
        };
        let result = client.get_option_depth(req).await;
        assert!(result.is_ok(), "get_option_depth should succeed: {:?}", result);
        let data: Vec<Depth> = result.unwrap();
        if !data.is_empty() {
            assert!(
                !data[0].asks.is_empty() || !data[0].bids.is_empty(),
                "OptionDepth should have asks or bids"
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_option_symbols() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = OptionSymbolsRequest {
            market: Some("HK".to_string()),
            ..Default::default()
        };
        let result = client.get_option_symbols(req).await;
        // HK option entitlements may not be present; only assert on Ok.
        if result.is_err() {
            return; // skip when entitlement unavailable
        }
        let data: Vec<OptionSymbol> = result.unwrap();
        if !data.is_empty() {
            let s = &data[0];
            assert!(
                !s.symbol.is_empty(),
                "OptionSymbol.symbol should be non-empty"
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_option_analysis() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = OptionAnalysisRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            period: Some("52week".to_string()),
            market: Some("US".to_string()),
            ..Default::default()
        };
        let result = client.get_option_analysis(req).await;
        assert!(
            result.is_ok(),
            "get_option_analysis should succeed: {:?}",
            result
        );
        let data: Vec<OptionAnalysis> = result.unwrap();
        assert!(
            !data.is_empty(),
            "option_analysis result should not be empty for AAPL"
        );
        let a = &data[0];
        assert_eq!(
            a.symbol, "AAPL",
            "OptionAnalysis.symbol should be AAPL, got {:?}",
            a.symbol
        );
    }

    // ── 期货行情 ──────────────────────────────────────────────────────────

    /// Helper: fetch first exchange code + first contract code for futures tests.
    async fn first_future_contract(client: &QuoteClient) -> Option<(String, String)> {
        let exchanges = client.get_future_exchange().await.ok()?;
        let ex = exchanges.first()?;
        let contracts = client
            .get_future_contracts(&ex.code)
            .await
            .ok()?;
        let c = contracts.first()?;
        Some((ex.code.clone(), c.contract_code.clone()))
    }

    #[tokio::test]
    async fn test_integ_get_future_exchange() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let result = client.get_future_exchange().await;
        // Futures entitlements may not be present; skip on error.
        if result.is_err() {
            return;
        }
        let data: Vec<FutureExchange> = result.unwrap();
        assert!(!data.is_empty(), "future_exchange list should not be empty");
        let e = &data[0];
        assert!(
            !e.code.is_empty(),
            "FutureExchange.code should be non-empty"
        );
        assert!(
            !e.name.is_empty(),
            "FutureExchange.name should be non-empty"
        );
    }

    #[tokio::test]
    async fn test_integ_get_future_contracts() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let exchanges = match client.get_future_exchange().await {
            Ok(e) if !e.is_empty() => e,
            _ => return,
        };
        let result = client.get_future_contracts(&exchanges[0].code).await;
        if result.is_err() {
            return;
        }
        let data: Vec<FutureContractInfo> = result.unwrap();
        assert!(!data.is_empty(), "future_contracts list should not be empty");
        let c = &data[0];
        assert!(
            !c.contract_code.is_empty(),
            "FutureContractInfo.contract_code should be non-empty"
        );
    }

    #[tokio::test]
    async fn test_integ_get_future_real_time_quote() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let (_ex_code, contract_code) = match first_future_contract(&client).await {
            Some(v) => v,
            None => return,
        };
        let req = FutureRealTimeQuoteRequest {
            contract_codes: Some(vec![contract_code.clone()]),
            ..Default::default()
        };
        let result = client.get_future_real_time_quote(req).await;
        if result.is_err() {
            return;
        }
        let data: Vec<FutureQuote> = result.unwrap();
        assert!(!data.is_empty(), "future_real_time_quote should not be empty");
        let q = &data[0];
        assert_eq!(
            q.contract_code, contract_code,
            "FutureQuote.contract_code mismatch"
        );
    }

    #[tokio::test]
    async fn test_integ_get_future_kline() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let (_ex_code, contract_code) = match first_future_contract(&client).await {
            Some(v) => v,
            None => return,
        };
        let req = FutureKlineRequest {
            contract_code: Some(contract_code.clone()),
            period: Some("day".to_string()),
            limit: Some(5),
            begin_time: Some(-1),
            end_time: Some(-1),
            ..Default::default()
        };
        let result = client.get_future_kline(req).await;
        if result.is_err() {
            return;
        }
        let data: Vec<FutureKline> = result.unwrap();
        assert!(!data.is_empty(), "future_kline result should not be empty");
        if !data[0].items.is_empty() {
            let item: &FutureKlineItem = &data[0].items[0];
            assert!(
                item.time > 0,
                "FutureKlineItem.time should be non-zero, got {}",
                item.time
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_future_contract() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let (_ex_code, contract_code) = match first_future_contract(&client).await {
            Some(v) => v,
            None => return,
        };
        let req = FutureContractSingleRequest {
            contract_code: Some(contract_code.clone()),
            ..Default::default()
        };
        let result = client.get_future_contract(req).await;
        if result.is_err() {
            return;
        }
        let data: Vec<FutureContractInfo> = result.unwrap();
        assert!(!data.is_empty(), "future_contract result should not be empty");
        assert_eq!(
            data[0].contract_code, contract_code,
            "FutureContractInfo.contract_code mismatch"
        );
    }

    #[tokio::test]
    async fn test_integ_get_all_future_contracts() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = AllFutureContractsRequest::default();
        let result = client.get_all_future_contracts(req).await;
        if result.is_err() {
            return;
        }
        let data: Vec<FutureContractInfo> = result.unwrap();
        if !data.is_empty() {
            assert!(
                !data[0].contract_code.is_empty(),
                "FutureContractInfo.contract_code should be non-empty"
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_current_future_contract() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let (_ex_code, contract_code) = match first_future_contract(&client).await {
            Some(v) => v,
            None => return,
        };
        let req = FutureContractSingleRequest {
            contract_code: Some(contract_code),
            ..Default::default()
        };
        let result = client.get_current_future_contract(req).await;
        if result.is_err() {
            return;
        }
        // May legitimately be None for non-continuous contracts.
        let _data: Option<FutureContractInfo> = result.unwrap();
    }

    #[tokio::test]
    async fn test_integ_get_future_continuous_contracts() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = FutureContinuousContractsRequest::default();
        let result = client.get_future_continuous_contracts(req).await;
        if result.is_err() {
            return;
        }
        let data: Vec<FutureContractInfo> = result.unwrap();
        if !data.is_empty() {
            assert!(
                !data[0].contract_code.is_empty(),
                "FutureContractInfo.contract_code should be non-empty"
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_future_history_main_contract() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let (_ex_code, contract_code) = match first_future_contract(&client).await {
            Some(v) => v,
            None => return,
        };
        let req = FutureHistoryMainContractRequest {
            contract_codes: Some(vec![contract_code.clone()]),
            ..Default::default()
        };
        let result = client.get_future_history_main_contract(req).await;
        if result.is_err() {
            return;
        }
        let data: Vec<FutureMainContractHistory> = result.unwrap();
        if !data.is_empty() {
            assert_eq!(
                data[0].contract_code, contract_code,
                "FutureMainContractHistory.contract_code mismatch"
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_future_kline_by_page() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let (_ex_code, contract_code) = match first_future_contract(&client).await {
            Some(v) => v,
            None => return,
        };
        let req = FutureKlineByPageRequest {
            contract_code: Some(contract_code),
            period: Some("day".to_string()),
            total_size: Some(5),
            page_size: Some(5),
            ..Default::default()
        };
        let result = client.get_future_kline_by_page(req).await;
        if result.is_err() {
            return;
        }
        let data: Vec<FutureKlineItem> = result.unwrap();
        if !data.is_empty() {
            assert!(
                data[0].time > 0,
                "FutureKlineItem.time should be non-zero, got {}",
                data[0].time
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_future_trade_ticks() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let (_ex_code, contract_code) = match first_future_contract(&client).await {
            Some(v) => v,
            None => return,
        };
        let req = FutureTradeTicksRequest {
            contract_code: Some(contract_code.clone()),
            limit: Some(5),
            ..Default::default()
        };
        let result = client.get_future_trade_ticks(req).await;
        if result.is_err() {
            return;
        }
        let data: Vec<FutureTradeTickItem> = result.unwrap();
        if !data.is_empty() {
            assert_eq!(
                data[0].contract_code, contract_code,
                "FutureTradeTickItem.contract_code mismatch"
            );
            assert!(
                data[0].time > 0,
                "FutureTradeTickItem.time should be non-zero, got {}",
                data[0].time
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_future_depth() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let (_ex_code, contract_code) = match first_future_contract(&client).await {
            Some(v) => v,
            None => return,
        };
        let req = FutureDepthRequest {
            contract_codes: Some(vec![contract_code.clone()]),
            ..Default::default()
        };
        let result = client.get_future_depth(req).await;
        if result.is_err() {
            return;
        }
        let data: Vec<FutureDepth> = result.unwrap();
        if !data.is_empty() {
            assert_eq!(
                data[0].contract_code, contract_code,
                "FutureDepth.contract_code mismatch"
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_future_trading_times() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let (_ex_code, contract_code) = match first_future_contract(&client).await {
            Some(v) => v,
            None => return,
        };
        let req = FutureTradingTimesRequest {
            contract_code: Some(contract_code),
            ..Default::default()
        };
        let result = client.get_future_trading_times(req).await;
        if result.is_err() {
            return;
        }
        let data: Option<FutureTradingTime> = result.unwrap();
        if let Some(t) = data {
            assert!(
                !t.contract_code.is_empty(),
                "FutureTradingTime.contract_code should be non-empty"
            );
        }
    }

    // ── 基金 ──────────────────────────────────────────────────────────────

    /// Helper: fetch first fund symbol from get_fund_symbols (not hardcoded).
    async fn first_fund_symbol(client: &QuoteClient) -> Option<String> {
        let req = FundSymbolsRequest::default();
        let data = client.get_fund_symbols(req).await.ok()?;
        data.into_iter().find(|s| !s.is_empty())
    }

    #[tokio::test]
    async fn test_integ_get_fund_symbols() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = FundSymbolsRequest::default();
        let result = client.get_fund_symbols(req).await;
        if result.is_err() {
            return; // fund entitlement may be unavailable
        }
        let data: Vec<String> = result.unwrap();
        if !data.is_empty() {
            assert!(
                data.iter().any(|s| !s.is_empty()),
                "fund symbol strings should be non-empty"
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_fund_contracts() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        // Dynamically fetch a fund symbol; skip if no entitlement or no symbols.
        let symbol = match first_fund_symbol(&client).await {
            Some(s) => s,
            None => return, // fund entitlement unavailable or no symbols
        };
        let req = FundContractsRequest {
            symbols: Some(vec![symbol.clone()]),
            ..Default::default()
        };
        let result = client.get_fund_contracts(req).await;
        if result.is_err() {
            return;
        }
        let data: Vec<FundContractInfo> = result.unwrap();
        if !data.is_empty() {
            assert_eq!(
                data[0].symbol, symbol,
                "FundContractInfo.symbol mismatch, got {:?}",
                data[0].symbol
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_fund_quote() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        // Dynamically fetch a fund symbol; skip if no entitlement or no symbols.
        let symbol = match first_fund_symbol(&client).await {
            Some(s) => s,
            None => return, // fund entitlement unavailable or no symbols
        };
        let req = FundQuoteRequest {
            symbols: Some(vec![symbol.clone()]),
            ..Default::default()
        };
        let result = client.get_fund_quote(req).await;
        if result.is_err() {
            return;
        }
        let data: Vec<FundQuote> = result.unwrap();
        if !data.is_empty() {
            assert_eq!(
                data[0].symbol, symbol,
                "FundQuote.symbol mismatch, got {:?}",
                data[0].symbol
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_fund_history_quote() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        // Dynamically fetch a fund symbol; skip if no entitlement or no symbols.
        let symbol = match first_fund_symbol(&client).await {
            Some(s) => s,
            None => return, // fund entitlement unavailable or no symbols
        };
        let req = FundHistoryQuoteRequest {
            symbols: Some(vec![symbol.clone()]),
            limit: Some(5),
            ..Default::default()
        };
        let result = client.get_fund_history_quote(req).await;
        if result.is_err() {
            return;
        }
        let data: Vec<FundHistoryQuote> = result.unwrap();
        if !data.is_empty() {
            assert_eq!(
                data[0].symbol, symbol,
                "FundHistoryQuote.symbol mismatch, got {:?}",
                data[0].symbol
            );
        }
    }

    // ── 窝轮（HK）──────────────────────────────────────────────────────────

    /// Helper: fetch first warrant symbol from get_warrant_filter (not hardcoded).
    /// Queries warrants for a well-known HK underlying (00700.HK / Tencent).
    async fn first_warrant_symbol(client: &QuoteClient) -> Option<String> {
        let req = WarrantFilterRequest {
            symbol: Some("00700.HK".to_string()),
            page: Some(0),
            page_size: Some(5),
            ..Default::default()
        };
        let result = client.get_warrant_filter(req).await.ok()??;
        // Extract first warrant symbol from the result items.
        result.items.first().map(|w| w.symbol.clone())
    }

    #[tokio::test]
    async fn test_integ_get_warrant_quote() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        // Dynamically fetch a warrant symbol; skip if no HK entitlement or no warrants.
        let symbol = match first_warrant_symbol(&client).await {
            Some(s) => s,
            None => return, // HK warrant entitlement unavailable or no warrants found
        };
        let req = WarrantQuoteRequest {
            symbols: Some(vec![symbol.clone()]),
            ..Default::default()
        };
        let result = client.get_warrant_quote(req).await;
        if result.is_err() {
            return;
        }
        let data: Vec<WarrantBrief> = result.unwrap();
        if !data.is_empty() {
            assert!(
                !data[0].symbol.is_empty(),
                "WarrantBrief.symbol should be non-empty"
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_warrant_filter() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = WarrantFilterRequest {
            symbol: Some("00700.HK".to_string()),
            page: Some(0),
            page_size: Some(5),
            ..Default::default()
        };
        let result = client.get_warrant_filter(req).await;
        if result.is_err() {
            return;
        }
        let data = result.unwrap();
        if let Some(r) = data {
            assert!(
                r.total >= 0,
                "WarrantFilterResult.total should be >= 0, got {}",
                r.total
            );
        }
    }

    // ── 行业 ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_integ_get_industry_list() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = IndustryListRequest {
            ..Default::default()
        };
        let result = client.get_industry_list(req).await;
        assert!(
            result.is_ok(),
            "get_industry_list should succeed: {:?}",
            result
        );
        let data: Vec<IndustryItem> = result.unwrap();
        assert!(!data.is_empty(), "industry_list should not be empty");
        let i = &data[0];
        assert!(
            !i.id.is_empty(),
            "IndustryItem.id should be non-empty"
        );
        assert!(
            !i.name.is_empty(),
            "IndustryItem.name should be non-empty"
        );
    }

    #[tokio::test]
    async fn test_integ_get_industry_stocks() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        // First fetch an industry id, then query its stocks.
        let list = client
            .get_industry_list(IndustryListRequest::default())
            .await
            .ok();
        let industry_id = match list {
            Some(v) if !v.is_empty() => v[0].id.clone(),
            _ => return,
        };
        let req = IndustryStocksRequest {
            industry_id: Some(industry_id),
            market: Some("US".to_string()),
            ..Default::default()
        };
        let result = client.get_industry_stocks(req).await;
        assert!(
            result.is_ok(),
            "get_industry_stocks should succeed: {:?}",
            result
        );
        let data: Vec<IndustryStock> = result.unwrap();
        if !data.is_empty() {
            assert!(
                !data[0].symbol.is_empty(),
                "IndustryStock.symbol should be non-empty"
            );
        }
    }

    // ── 公司行动子接口 ───────────────────────────────────────────────────

    #[tokio::test]
    async fn test_integ_get_corporate_split() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = qm::CorporateActionRequest {
            symbols: vec!["AAPL".to_string()],
            market: "US".to_string(),
            action_type: String::new(),
            begin_date: "2020-01-01".to_string(),
            end_date: "2025-12-31".to_string(),
        };
        let result = client.get_corporate_split(req).await;
        assert!(
            result.is_ok(),
            "get_corporate_split should succeed: {:?}",
            result
        );
        let data: Vec<CorporateAction> = result.unwrap();
        if !data.is_empty() {
            let ca = &data[0];
            assert_eq!(ca.symbol, "AAPL");
        }
    }

    #[tokio::test]
    async fn test_integ_get_corporate_dividend() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = qm::CorporateActionRequest {
            symbols: vec!["AAPL".to_string()],
            market: "US".to_string(),
            action_type: String::new(),
            begin_date: "2024-01-01".to_string(),
            end_date: "2025-12-31".to_string(),
        };
        let result = client.get_corporate_dividend(req).await;
        assert!(
            result.is_ok(),
            "get_corporate_dividend should succeed: {:?}",
            result
        );
        let data: Vec<CorporateAction> = result.unwrap();
        assert!(
            !data.is_empty(),
            "dividend result should not be empty for AAPL"
        );
        assert_eq!(data[0].symbol, "AAPL");
    }

    #[tokio::test]
    async fn test_integ_get_corporate_earnings_calendar() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = qm::CorporateActionRequest {
            symbols: vec!["AAPL".to_string()],
            market: "US".to_string(),
            action_type: String::new(),
            begin_date: "2024-01-01".to_string(),
            end_date: "2025-12-31".to_string(),
        };
        let result = client.get_corporate_earnings_calendar(req).await;
        assert!(
            result.is_ok(),
            "get_corporate_earnings_calendar should succeed: {:?}",
            result
        );
        let data: Vec<CorporateAction> = result.unwrap();
        if !data.is_empty() {
            assert_eq!(data[0].symbol, "AAPL");
        }
    }

    #[tokio::test]
    async fn test_integ_get_corporate_symbol_change() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = qm::CorporateActionRequest {
            symbols: vec!["AAPL".to_string()],
            market: "US".to_string(),
            action_type: String::new(),
            begin_date: "2020-01-01".to_string(),
            end_date: "2025-12-31".to_string(),
        };
        let result = client.get_corporate_symbol_change(req).await;
        assert!(
            result.is_ok(),
            "get_corporate_symbol_change should succeed: {:?}",
            result
        );
        let _data = result.unwrap();
    }

    #[tokio::test]
    async fn test_integ_get_corporate_delisting() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = qm::CorporateActionRequest {
            symbols: vec!["AAPL".to_string()],
            market: "US".to_string(),
            action_type: String::new(),
            begin_date: "2020-01-01".to_string(),
            end_date: "2025-12-31".to_string(),
        };
        let result = client.get_corporate_delisting(req).await;
        assert!(
            result.is_ok(),
            "get_corporate_delisting should succeed: {:?}",
            result
        );
        let _data = result.unwrap();
    }

    #[tokio::test]
    async fn test_integ_get_corporate_ipo() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = qm::CorporateActionRequest {
            symbols: vec!["AAPL".to_string()],
            market: "US".to_string(),
            action_type: String::new(),
            begin_date: "2020-01-01".to_string(),
            end_date: "2025-12-31".to_string(),
        };
        let result = client.get_corporate_ipo(req).await;
        assert!(
            result.is_ok(),
            "get_corporate_ipo should succeed: {:?}",
            result
        );
        let _data = result.unwrap();
    }

    // ── 财务 / 日历 / 资金分布 / 扫描 / 隔夜 / token ────────────────────

    #[tokio::test]
    async fn test_integ_get_financial_daily() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = qm::FinancialDailyRequest {
            symbols: vec!["AAPL".to_string()],
            market: "US".to_string(),
            fields: vec!["shares_outstanding".to_string()],
            begin_date: "2024-01-01".to_string(),
            end_date: "2024-06-30".to_string(),
        };
        let result = client.get_financial_daily(req).await;
        assert!(
            result.is_ok(),
            "get_financial_daily should succeed: {:?}",
            result
        );
        let data = result.unwrap();
        if !data.is_empty() {
            assert_eq!(data[0].symbol, "AAPL");
        }
    }

    #[tokio::test]
    async fn test_integ_get_financial_report() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = qm::FinancialReportRequest {
            symbols: vec!["AAPL".to_string()],
            market: "US".to_string(),
            fields: vec!["total_revenue".to_string()],
            period_type: "ANNUAL".to_string(),
            begin_date: String::new(),
            end_date: String::new(),
        };
        let result = client.get_financial_report(req).await;
        assert!(
            result.is_ok(),
            "get_financial_report should succeed: {:?}",
            result
        );
        let data = result.unwrap();
        if !data.is_empty() {
            assert_eq!(data[0].symbol, "AAPL");
        }
    }

    #[tokio::test]
    async fn test_integ_get_financial_currency() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = FinancialCurrencyRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            market: Some("US".to_string()),
            ..Default::default()
        };
        let result = client.get_financial_currency(req).await;
        assert!(
            result.is_ok(),
            "get_financial_currency should succeed: {:?}",
            result
        );
        let data: Vec<FinancialCurrency> = result.unwrap();
        if !data.is_empty() {
            assert_eq!(data[0].symbol, "AAPL");
        }
    }

    #[tokio::test]
    async fn test_integ_get_financial_exchange_rate() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = FinancialExchangeRateRequest {
            currency_list: Some(vec!["USD/CNY".to_string()]),
            begin_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-01-31".to_string()),
            ..Default::default()
        };
        let result = client.get_financial_exchange_rate(req).await;
        assert!(
            result.is_ok(),
            "get_financial_exchange_rate should succeed: {:?}",
            result
        );
        let data: Vec<ExchangeRate> = result.unwrap();
        if !data.is_empty() {
            assert!(
                !data[0].currency.is_empty(),
                "ExchangeRate.currency should be non-empty"
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_trading_calendar() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = TradingCalendarRequest {
            market: Some("US".to_string()),
            begin_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-01-31".to_string()),
            ..Default::default()
        };
        let result = client.get_trading_calendar(req).await;
        assert!(
            result.is_ok(),
            "get_trading_calendar should succeed: {:?}",
            result
        );
        let data: Vec<TradingCalendarItem> = result.unwrap();
        assert!(
            !data.is_empty(),
            "trading_calendar should not be empty for US Jan 2024"
        );
        let c = &data[0];
        assert_eq!(c.market, "US");
        assert!(!c.date.is_empty());
    }

    #[tokio::test]
    async fn test_integ_get_capital_distribution() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let result = client.get_capital_distribution("AAPL", "US").await;
        assert!(
            result.is_ok(),
            "get_capital_distribution should succeed: {:?}",
            result
        );
        let data: Option<CapitalDistribution> = result.unwrap();
        if let Some(d) = data {
            assert_eq!(d.symbol, "AAPL");
            assert!(d.in_all >= 0.0);
        }
    }

    #[tokio::test]
    async fn test_integ_market_scanner() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = qm::MarketScannerRequest {
            market: "US".to_string(),
            page: Some(0),
            page_size: Some(5),
            ..Default::default()
        };
        let result = client.market_scanner(req).await;
        assert!(result.is_ok(), "market_scanner should succeed: {:?}", result);
        let data = result.unwrap();
        if let Some(r) = data {
            assert!(r.page >= 0);
            assert!(r.page_size >= 0);
        }
    }

    #[tokio::test]
    async fn test_integ_get_market_scanner_tags() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = MarketScannerTagsRequest {
            market: Some("US".to_string()),
            ..Default::default()
        };
        let result = client.get_market_scanner_tags(req).await;
        assert!(
            result.is_ok(),
            "get_market_scanner_tags should succeed: {:?}",
            result
        );
        let data: Vec<MarketScannerTagGroup> = result.unwrap();
        if !data.is_empty() {
            assert!(
                !data[0].market.is_empty(),
                "MarketScannerTagGroup.market should be non-empty"
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_quote_overnight() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = QuoteOvernightRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        };
        let result = client.get_quote_overnight(req).await;
        assert!(
            result.is_ok(),
            "get_quote_overnight should succeed: {:?}",
            result
        );
        let data: Vec<QuoteOvernight> = result.unwrap();
        if !data.is_empty() {
            assert_eq!(data[0].symbol, "AAPL");
        }
    }

    #[tokio::test]
    async fn test_integ_query_token() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let result = client.query_token().await;
        assert!(result.is_ok(), "query_token should succeed: {:?}", result);
        let token = result.unwrap();
        assert!(
            !token.is_empty(),
            "refreshed token string should be non-empty"
        );
    }
}
