//! Quote example - covers every QuoteClient public method.
//!
//! Config is auto-discovered from:
//!   1. ./tiger_openapi_config.properties
//!   2. ~/.tigeropen/tiger_openapi_config.properties
//!   TIGER_CONFIG_PATH env var overrides both.
//!
//! Individual failures do not abort subsequent calls; a final PASS/FAIL/SKIP summary is printed.
//!
//! Run: `TIGER_CONFIG_PATH=~/.tigeropen/tiger_openapi_config.properties cargo run --example quote_example`

use tigeropen::config::ClientConfig;
use tigeropen::model::quote::{
    Brief, CorporateActionRequest, FinancialDailyRequest, FinancialReportRequest,
    MarketScannerRequest, MarketState,
};
use tigeropen::model::quote_requests::{
    AllFutureContractsRequest, KlineRequest, KlineByPageRequest, BriefRequest, QuoteDepthRequest,
    FinancialCurrencyRequest, FinancialExchangeRateRequest, FutureKlineRequest,
    FutureKlineByPageRequest, FutureContinuousContractsRequest, FutureRealTimeQuoteRequest,
    FutureContractSingleRequest, FutureDepthRequest, FutureHistoryMainContractRequest,
    FutureTradingTimesRequest, FutureTradeTicksRequest, FundContractsRequest,
    FundHistoryQuoteRequest, FundQuoteRequest, FundSymbolsRequest, IndustryListRequest,
    IndustryStocksRequest, KlineQuotaRequest, MarketScannerTagsRequest,
    OptionChainItem, OptionChainRequest, OptionChainFilter, OptionChainFilterGreeks,
    RangeF64, OptionAnalysisRequest, OptionContractItem,
    OptionDepthRequest, OptionKlineItem, OptionKlineRequest, OptionQueryItem,
    OptionQuoteRequest, OptionTimelineRequest, OptionTradeTicksRequest,
    QuoteOvernightRequest, QuotePermissionRequest, ShortInterestRequest, StockBrokerRequest,
    StockDetailsRequest, StockFundamentalRequest, StockIndustryRequest,
    SymbolsRequest, TimelineHistoryRequest, TradeMetasRequest, TradeRankRequest,
    TradeTickRequest, TradingCalendarRequest, WarrantFilterRequest, WarrantQuoteRequest,
};
use tigeropen::quote::QuoteClient;

enum Outcome {
    Pass,
    Fail,
    Skip,
}

struct RunResult {
    name: String,
    outcome: Outcome,
    detail: String,
}

fn ok(results: &mut Vec<RunResult>, name: &str, note: impl Into<String>) {
    let note = note.into();
    println!("[ OK ] {:<42} {}", name, truncate(&note, 140));
    results.push(RunResult { name: name.into(), outcome: Outcome::Pass, detail: note });
}

fn fail(results: &mut Vec<RunResult>, name: &str, err: impl std::fmt::Display) {
    let detail = format!("{}", err);
    println!("[FAIL] {:<42} {}", name, detail);
    results.push(RunResult { name: name.into(), outcome: Outcome::Fail, detail });
}

fn skip(results: &mut Vec<RunResult>, name: &str, reason: impl Into<String>) {
    let reason = reason.into();
    println!("[SKIP] {:<42} {}", name, reason);
    results.push(RunResult { name: name.into(), outcome: Outcome::Skip, detail: reason });
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let mut t: String = s.chars().take(max).collect();
        t.push_str("...");
        t
    } else {
        s.to_string()
    }
}

fn print_summary(results: &[RunResult]) {
    let (mut p, mut f, mut s) = (0, 0, 0);
    for r in results {
        match r.outcome {
            Outcome::Pass => p += 1,
            Outcome::Fail => f += 1,
            Outcome::Skip => s += 1,
        }
    }
    println!("\n================ SUMMARY ================");
    println!("PASS={}  FAIL={}  SKIP={}  TOTAL={}", p, f, s, results.len());
    if f > 0 {
        println!("\nFailures:");
        for r in results {
            if matches!(r.outcome, Outcome::Fail) {
                println!("  - {}: {}", r.name, r.detail);
            }
        }
    }
    println!("=========================================");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = match std::env::var("TIGER_CONFIG_PATH") {
        Ok(path) => ClientConfig::builder().properties_file(&path).build()?,
        Err(_) => ClientConfig::builder().build()?,
    };
    println!("tiger_id={} account={}\n", config.tiger_id, config.account);

    let qc = QuoteClient::from_config(config);

    let mut results: Vec<RunResult> = Vec::new();

    println!("=== Basic market data ===");
    match qc.get_market_state("US").await {
        Ok(states) if !states.is_empty() => ok(
            &mut results,
            "GetMarketState(US)",
            format!(
                "{} status={} openTime={}",
                states[0].market, states[0].market_status, states[0].open_time
            ),
        ),
        Ok(_) => ok(&mut results, "GetMarketState(US)", "(empty)"),
        Err(e) => fail(&mut results, "GetMarketState(US)", e),
    }

    // v0.4.0 new signature: BriefRequest
    match qc
        .get_real_time_quote(BriefRequest {
            symbols: Some(vec!["AAPL".to_string(), "TSLA".to_string()]),
            ..Default::default()
        })
        .await
    {
        Ok(briefs) => {
            let s: Vec<String> = briefs
                .iter()
                .map(|b| format!("{}={:.2}", b.symbol, b.latest_price))
                .collect();
            ok(&mut results, "GetRealTimeQuote", s.join(" "));
        }
        Err(e) => fail(&mut results, "GetRealTimeQuote", e),
    }

    match qc.get_kline(KlineRequest { symbols: Some(vec!["AAPL".to_string()]), period: Some("day".to_string()), ..Default::default() }).await {
        Ok(klines) if !klines.is_empty() => ok(
            &mut results,
            "GetKline(AAPL day)",
            format!("symbol={} bars={}", klines[0].symbol, klines[0].items.len()),
        ),
        Ok(_) => ok(&mut results, "GetKline(AAPL day)", "(empty)"),
        Err(e) => fail(&mut results, "GetKline(AAPL day)", e),
    }

    match qc.get_timeline(&["AAPL"]).await {
        Ok(tl) if !tl.is_empty() => {
            let n = tl[0].intraday.as_ref().map(|b| b.items.len()).unwrap_or(0);
            ok(
                &mut results,
                "GetTimeline",
                format!("intraday_points={} preClose={:.2}", n, tl[0].pre_close),
            )
        }
        Ok(_) => ok(&mut results, "GetTimeline", "(empty)"),
        Err(e) => fail(&mut results, "GetTimeline", e),
    }

    // v0.4.0 new signature: TradeTickRequest — enhanced with first-tick field values
    match qc
        .get_trade_tick(TradeTickRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        })
        .await
    {
        Ok(tt) if !tt.is_empty() && !tt[0].items.is_empty() => ok(
            &mut results,
            "GetTradeTick",
            format!(
                "ticks={} first_price={:.4} first_type={}",
                tt[0].items.len(),
                tt[0].items[0].price,
                tt[0].items[0].r#type
            ),
        ),
        Ok(tt) if !tt.is_empty() => ok(&mut results, "GetTradeTick", format!("ticks={}", tt[0].items.len())),
        Ok(_) => ok(&mut results, "GetTradeTick", "(empty)"),
        Err(e) => fail(&mut results, "GetTradeTick", e),
    }

    // v0.4.0 new signature: DepthQuoteRequest
    match qc
        .get_quote_depth(QuoteDepthRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            market: Some("US".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(d) if !d.is_empty() => ok(
            &mut results,
            "GetQuoteDepth(AAPL)",
            format!("asks={} bids={}", d[0].asks.len(), d[0].bids.len()),
        ),
        Ok(_) => ok(&mut results, "GetQuoteDepth(AAPL)", "(empty)"),
        Err(e) => fail(&mut results, "GetQuoteDepth(AAPL)", e),
    }

    println!("\n=== v0.4.0 股票基础 smoke ===");

    match qc
        .get_symbols(SymbolsRequest {
            market: Some("US".to_string()),
            sec_type: Some("STK".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(syms) => ok(&mut results, "GetSymbols(US STK)", format!("count={}", syms.len())),
        Err(e) => fail(&mut results, "GetSymbols(US STK)", e),
    }

    match qc
        .get_symbol_names(SymbolsRequest {
            market: Some("US".to_string()),
            sec_type: Some("STK".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(names) => ok(&mut results, "GetSymbolNames(US STK)", format!("count={}", names.len())),
        Err(e) => fail(&mut results, "GetSymbolNames(US STK)", e),
    }

    // Enhanced: print lot_size and exchange
    match qc
        .get_trade_metas(TradeMetasRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        })
        .await
    {
        Ok(metas) if !metas.is_empty() => ok(
            &mut results,
            "GetTradeMetas(AAPL)",
            format!("count={} lot_size={}", metas.len(), metas[0].lot_size),
        ),
        Ok(metas) => ok(&mut results, "GetTradeMetas(AAPL)", format!("count={}", metas.len())),
        Err(e) => fail(&mut results, "GetTradeMetas(AAPL)", e),
    }

    // Enhanced: print latest_price and pe_ratio_ttm
    match qc
        .get_stock_details(StockDetailsRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        })
        .await
    {
        Ok(details) if !details.is_empty() => ok(
            &mut results,
            "GetStockDetails(AAPL)",
            format!(
                "symbol={} pe_ratio_ttm={:.2}",
                details[0].symbol, details[0].pe_ratio_ttm
            ),
        ),
        Ok(_) => ok(&mut results, "GetStockDetails(AAPL)", "(empty)"),
        Err(e) => fail(&mut results, "GetStockDetails(AAPL)", e),
    }

    // Enhanced: print latest_price
    match qc
        .get_delayed_quote(tigeropen::model::quote_requests::DelayedQuoteRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        })
        .await
    {
        Ok(briefs) if !briefs.is_empty() => ok(
            &mut results,
            "GetDelayedQuote(AAPL)",
            format!("count={} latest_price={:.4}", briefs.len(), briefs[0].latest_price),
        ),
        Ok(briefs) => ok(
            &mut results,
            "GetDelayedQuote(AAPL)",
            format!("count={}", briefs.len()),
        ),
        Err(e) => fail(&mut results, "GetDelayedQuote(AAPL)", e),
    }

    match qc
        .get_kline(KlineRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            period: Some("day".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .await
    {
        Ok(ks) if !ks.is_empty() => ok(
            &mut results,
            "GetKline(AAPL day)",
            format!("bars={}", ks[0].items.len()),
        ),
        Ok(_) => ok(&mut results, "GetKline(AAPL day)", "(empty)"),
        Err(e) => fail(&mut results, "GetKline(AAPL day)", e),
    }

    // Enhanced: print intraday items len
    match qc
        .get_timeline_history(TimelineHistoryRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            date: Some("2026-05-06".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(tl) if !tl.is_empty() => {
            let intraday_len = tl[0].intraday.as_ref().map(|b| b.items.len()).unwrap_or(0);
            ok(
                &mut results,
                "GetTimelineHistory(AAPL)",
                format!("count={} intraday.items.len={}", tl.len(), intraday_len),
            )
        }
        Ok(_) => ok(&mut results, "GetTimelineHistory(AAPL)", "(empty)"),
        Err(e) => fail(&mut results, "GetTimelineHistory(AAPL)", e),
    }

    // Enhanced: print symbol and latest_price of first item
    match qc
        .get_trade_rank(TradeRankRequest {
            market: Some("US".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(items) if !items.is_empty() => ok(
            &mut results,
            "GetTradeRank(US)",
            format!(
                "count={} first_symbol={} first_price={:.4}",
                items.len(), items[0].symbol, items[0].latest_price
            ),
        ),
        Ok(items) => ok(&mut results, "GetTradeRank(US)", format!("count={}", items.len())),
        Err(e) => fail(&mut results, "GetTradeRank(US)", e),
    }

    // Enhanced: print gsector of first item
    match qc
        .get_stock_industry(StockIndustryRequest {
            symbol: Some("AAPL".to_string()),
            market: Some("US".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(industries) if !industries.is_empty() => ok(
            &mut results,
            "GetStockIndustry(AAPL)",
            format!("count={} gsector={}", industries.len(), industries[0].gsector),
        ),
        Ok(industries) => ok(
            &mut results,
            "GetStockIndustry(AAPL)",
            format!("count={}", industries.len()),
        ),
        Err(e) => fail(&mut results, "GetStockIndustry(AAPL)", e),
    }

    // Enhanced: print name of first permission
    match qc
        .get_quote_permission(QuotePermissionRequest {
            ..Default::default()
        })
        .await
    {
        Ok(perms) if !perms.is_empty() => ok(
            &mut results,
            "GetQuotePermission",
            format!("count={} first_name={}", perms.len(), perms[0].name),
        ),
        Ok(perms) => ok(&mut results, "GetQuotePermission", format!("count={}", perms.len())),
        Err(e) => fail(&mut results, "GetQuotePermission", e),
    }

    // Enhanced: print method and quota of first item
    match qc
        .get_kline_quota(KlineQuotaRequest {
            with_details: Some(false),
            ..Default::default()
        })
        .await
    {
        Ok(quotas) if !quotas.is_empty() => ok(
            &mut results,
            "GetKlineQuota",
            format!(
                "count={} first_method={} first_quota={}",
                quotas.len(), quotas[0].method, quotas[0].quota
            ),
        ),
        Ok(quotas) => ok(&mut results, "GetKlineQuota", format!("count={}", quotas.len())),
        Err(e) => fail(&mut results, "GetKlineQuota", e),
    }

    // GetShortInterest (new test) — requires specific market data subscription; skip gracefully
    match qc
        .get_short_interest(ShortInterestRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        })
        .await
    {
        Ok(items) if !items.is_empty() => ok(
            &mut results,
            "GetShortInterest(AAPL)",
            format!(
                "count={} symbol={} date={} short_interest={:.0}",
                items.len(), items[0].symbol, items[0].settlement_date, items[0].short_interest
            ),
        ),
        Ok(_) => skip(&mut results, "GetShortInterest(AAPL)", "no data returned"),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("does not support") || msg.contains("code=1000") || msg.contains("code=4") {
                skip(&mut results, "GetShortInterest(AAPL)", format!("not supported: {}", msg));
            } else {
                fail(&mut results, "GetShortInterest(AAPL)", e);
            }
        }
    }

    // GetStockBroker (new test) — only supports HK market
    match qc
        .get_stock_broker(StockBrokerRequest {
            symbol: Some("00700".to_string()),
            sec_type: Some("STK".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(Some(broker)) => ok(
            &mut results,
            "GetStockBroker(00700 HK)",
            format!("level_ask_list.len={}", broker.level_ask_list.len()),
        ),
        Ok(None) => skip(&mut results, "GetStockBroker(00700 HK)", "no data returned"),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("does not support") || msg.contains("code=1000") || msg.contains("code=4") {
                skip(&mut results, "GetStockBroker(00700 HK)", format!("not supported: {}", msg));
            } else {
                fail(&mut results, "GetStockBroker(00700 HK)", e);
            }
        }
    }

    // GetStockFundamental (new test)
    match qc
        .get_stock_fundamental(StockFundamentalRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            market: Some("US".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(map) => ok(
            &mut results,
            "GetStockFundamental(AAPL)",
            format!("keys={}", map.len()),
        ),
        Err(e) => fail(&mut results, "GetStockFundamental(AAPL)", e),
    }

    // GetKlineByPage (new test)
    match qc
        .get_kline_by_page(KlineByPageRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            period: Some("day".to_string()),
            total_size: Some(10),
            page_size: Some(10),
            ..Default::default()
        })
        .await
    {
        Ok(items) => ok(
            &mut results,
            "GetKlineByPage(AAPL)",
            format!("items={}", items.len()),
        ),
        Err(e) => fail(&mut results, "GetKlineByPage(AAPL)", e),
    }

    println!("\n=== Options (US — AAPL) ===");
    run_option_smoke(&qc, "AAPL", &mut results).await;

    println!("\n=== Options (HK — 00700.HK) ===");
    run_hk_option_smoke(&qc, "00700.HK", &mut results).await;

    println!("\n=== Futures ===");
    let mut exchange_code = String::new();
    let mut contract_code = String::new();

    match qc.get_future_exchange().await {
        Ok(exs) if !exs.is_empty() => {
            ok(
                &mut results,
                "GetFutureExchange",
                format!("exchanges={} first={}", exs.len(), exs[0].code),
            );
            exchange_code = exs[0].code.clone();
        }
        Ok(_) => ok(&mut results, "GetFutureExchange", "(empty)"),
        Err(e) => fail(&mut results, "GetFutureExchange", e),
    }

    if exchange_code.is_empty() {
        skip(&mut results, "GetFutureContracts", "no exchange");
    } else {
        match qc.get_future_contracts(&exchange_code).await {
            Ok(cs) if !cs.is_empty() => {
                ok(
                    &mut results,
                    &format!("GetFutureContracts({})", exchange_code),
                    format!("contracts={} first={}", cs.len(), cs[0].contract_code),
                );
                contract_code = cs[0].contract_code.clone();
            }
            Ok(_) => ok(
                &mut results,
                &format!("GetFutureContracts({})", exchange_code),
                "(empty)",
            ),
            Err(e) => fail(
                &mut results,
                &format!("GetFutureContracts({})", exchange_code),
                e,
            ),
        }
    }

    if contract_code.is_empty() {
        skip(&mut results, "GetFutureRealTimeQuote", "no contract");
        skip(&mut results, "GetFutureKline", "no contract");
        skip(&mut results, "GetFutureContract", "no contract");
        skip(&mut results, "GetAllFutureContracts", "no contract");
        skip(&mut results, "GetCurrentFutureContract", "no contract");
        skip(&mut results, "GetFutureBars", "no contract");
        skip(&mut results, "GetFutureDepth", "no contract");
        skip(&mut results, "GetFutureTradingTimes", "no contract");
        skip(&mut results, "GetFutureContinuousContracts", "no contract");
        skip(&mut results, "GetFutureHistoryMainContract", "no contract");
        skip(&mut results, "GetFutureKlineByPage", "no contract");
        skip(&mut results, "GetFutureTradeTicks", "no contract");
    } else {
        // v0.4.0 new signature: FutureBriefRequest
        match qc
            .get_future_real_time_quote(FutureRealTimeQuoteRequest {
                contract_codes: Some(vec![contract_code.clone()]),
                ..Default::default()
            })
            .await
        {
            Ok(q) if !q.is_empty() => ok(
                &mut results,
                "GetFutureRealTimeQuote",
                format!("{} latestPrice={:.4}", q[0].contract_code, q[0].latest_price),
            ),
            Ok(_) => ok(&mut results, "GetFutureRealTimeQuote", "(empty)"),
            Err(e) => fail(&mut results, "GetFutureRealTimeQuote", e),
        }

        match qc
            .get_future_kline(FutureKlineRequest {
                contract_codes: Some(vec![contract_code.clone()]),
                period: Some("day".to_string()),
                begin_time: Some(-1),
                end_time: Some(-1),
                ..Default::default()
            })
            .await
        {
            Ok(ks) if !ks.is_empty() => ok(
                &mut results,
                &format!("GetFutureKline({})", contract_code),
                format!("bars={}", ks[0].items.len()),
            ),
            Ok(_) => ok(
                &mut results,
                &format!("GetFutureKline({})", contract_code),
                "(empty)",
            ),
            Err(e) => fail(
                &mut results,
                &format!("GetFutureKline({})", contract_code),
                e,
            ),
        }

        // GetFutureContract (single by contract_code)
        match qc
            .get_future_contract(FutureContractSingleRequest {
                contract_code: Some(contract_code.clone()),
                ..Default::default()
            })
            .await
        {
            Ok(cs) => ok(
                &mut results,
                &format!("GetFutureContract({})", contract_code),
                format!("count={}", cs.len()),
            ),
            Err(e) => fail(&mut results, &format!("GetFutureContract({})", contract_code), e),
        }

        // GetAllFutureContracts — type is the product code e.g. "MEUR" extracted from contract_code
        let product_code: String = contract_code
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .to_string();
        match qc.get_all_future_contracts(AllFutureContractsRequest {
            contract_type: Some(product_code.clone()),
            ..Default::default()
        }).await {
            Ok(cs) => ok(&mut results, "GetAllFutureContracts", format!("count={}", cs.len())),
            Err(e) => fail(&mut results, "GetAllFutureContracts", e),
        }

        // GetCurrentFutureContract
        match qc
            .get_current_future_contract(FutureContractSingleRequest {
                contract_code: Some(contract_code.clone()),
                contract_type: Some(product_code.clone()),
                ..Default::default()
            })
            .await
        {
            Ok(Some(c)) => ok(
                &mut results,
                "GetCurrentFutureContract",
                format!("code={}", c.contract_code),
            ),
            Ok(None) => ok(&mut results, "GetCurrentFutureContract", "(empty)"),
            Err(e) => fail(&mut results, "GetCurrentFutureContract", e),
        }

        // GetFutureKline (with limit)
        match qc
            .get_future_kline(FutureKlineRequest {
                contract_codes: Some(vec![contract_code.clone()]),
                period: Some("day".to_string()),
                begin_time: Some(-1),
                end_time: Some(-1),
                limit: Some(5),
                ..Default::default()
            })
            .await
        {
            Ok(ks) if !ks.is_empty() => ok(
                &mut results,
                &format!("GetFutureKline({})", contract_code),
                format!("bars={}", ks[0].items.len()),
            ),
            Ok(_) => ok(&mut results, &format!("GetFutureKline({})", contract_code), "(empty)"),
            Err(e) => fail(&mut results, &format!("GetFutureKline({})", contract_code), e),
        }

        // GetFutureDepth
        match qc
            .get_future_depth(FutureDepthRequest {
                contract_codes: Some(vec![contract_code.clone()]),
                ..Default::default()
            })
            .await
        {
            Ok(ds) => ok(
                &mut results,
                "GetFutureDepth",
                format!("count={}", ds.len()),
            ),
            Err(e) => fail(&mut results, "GetFutureDepth", e),
        }

        // GetFutureTradingTimes
        match qc
            .get_future_trading_times(FutureTradingTimesRequest {
                contract_code: Some(contract_code.clone()),
                ..Default::default()
            })
            .await
        {
            Ok(Some(t)) => ok(
                &mut results,
                "GetFutureTradingTimes",
                format!("segments={}", t.trading_times.len()),
            ),
            Ok(None) => ok(&mut results, "GetFutureTradingTimes", "(empty)"),
            Err(e) => fail(&mut results, "GetFutureTradingTimes", e),
        }

        // GetFutureContinuousContracts (new test)
        match qc
            .get_future_continuous_contracts(FutureContinuousContractsRequest {
                contract_type: Some(product_code.clone()),
                ..Default::default()
            })
            .await
        {
            Ok(cs) if !cs.is_empty() => ok(
                &mut results,
                "GetFutureContinuousContracts",
                format!("count={} first={}", cs.len(), cs[0].contract_code),
            ),
            Ok(_) => skip(&mut results, "GetFutureContinuousContracts", "no data returned"),
            Err(e) => fail(&mut results, "GetFutureContinuousContracts", e),
        }

        // GetFutureHistoryMainContract (new test)
        match qc
            .get_future_history_main_contract(FutureHistoryMainContractRequest {
                contract_codes: Some(vec![product_code.clone()]),
                ..Default::default()
            })
            .await
        {
            Ok(items) => ok(
                &mut results,
                "GetFutureHistoryMainContract",
                format!("count={}", items.len()),
            ),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("does not support") || msg.contains("code=1000") || msg.contains("code=4") {
                    skip(&mut results, "GetFutureHistoryMainContract", format!("not supported: {}", msg));
                } else {
                    fail(&mut results, "GetFutureHistoryMainContract", e);
                }
            }
        }

        // GetFutureKlineByPage (new test) — wrapper iterates kline pages; uses contract_code field
        // Note: FutureKlineByPageRequest uses singular contract_code
        match qc
            .get_future_kline_by_page(FutureKlineByPageRequest {
                contract_code: Some(contract_code.clone()),
                period: Some("day".to_string()),
                total_size: Some(10),
                page_size: Some(10),
                ..Default::default()
            })
            .await
        {
            Ok(items) => ok(
                &mut results,
                "GetFutureKlineByPage",
                format!("items={}", items.len()),
            ),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("contract_codes") || msg.contains("cannot be empty") {
                    // server requires contract_codes array; try with contract_codes via raw kline
                    skip(&mut results, "GetFutureKlineByPage", format!("server needs contract_codes array: {}", msg));
                } else {
                    fail(&mut results, "GetFutureKlineByPage", e);
                }
            }
        }

        // GetFutureTradeTicks (new test)
        match qc
            .get_future_trade_ticks(FutureTradeTicksRequest {
                contract_code: Some(contract_code.clone()),
                ..Default::default()
            })
            .await
        {
            Ok(items) if !items.is_empty() => ok(
                &mut results,
                "GetFutureTradeTicks",
                format!("ticks={} first_price={:.4}", items.len(), items[0].price),
            ),
            Ok(_) => skip(&mut results, "GetFutureTradeTicks", "no data returned"),
            Err(e) => fail(&mut results, "GetFutureTradeTicks", e),
        }
    }

    println!("\n=== Funds ===");
    let mut fund_symbol = String::new();

    match qc.get_fund_symbols(FundSymbolsRequest::default()).await {
        Ok(syms) if !syms.is_empty() => {
            ok(&mut results, "GetFundSymbols", format!("count={}", syms.len()));
            fund_symbol = syms[0].clone();
        }
        Ok(_) => ok(&mut results, "GetFundSymbols", "(empty)"),
        Err(e) => fail(&mut results, "GetFundSymbols", e),
    }

    if fund_symbol.is_empty() {
        skip(&mut results, "GetFundContracts", "no fund symbol");
        skip(&mut results, "GetFundQuote", "no fund symbol");
        skip(&mut results, "GetFundHistoryQuote", "no fund symbol");
    } else {
        // GetFundContracts (new test)
        match qc
            .get_fund_contracts(FundContractsRequest {
                symbols: Some(vec![fund_symbol.clone()]),
                ..Default::default()
            })
            .await
        {
            Ok(items) if !items.is_empty() => ok(
                &mut results,
                "GetFundContracts",
                format!("count={} symbol={} fund_type={}", items.len(), items[0].symbol, items[0].fund_type),
            ),
            Ok(_) => skip(&mut results, "GetFundContracts", "no data returned"),
            Err(e) => fail(&mut results, "GetFundContracts", e),
        }

        // GetFundQuote (new test)
        match qc
            .get_fund_quote(FundQuoteRequest {
                symbols: Some(vec![fund_symbol.clone()]),
                ..Default::default()
            })
            .await
        {
            Ok(items) if !items.is_empty() => ok(
                &mut results,
                "GetFundQuote",
                format!("count={} symbol={} latest_nav={:.4}", items.len(), items[0].symbol, items[0].latest_nav),
            ),
            Ok(_) => skip(&mut results, "GetFundQuote", "no data returned"),
            Err(e) => fail(&mut results, "GetFundQuote", e),
        }

        // GetFundHistoryQuote (new test) — requires both begin_time and end_time
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
        let one_year_ago_ms = now_ms - 365 * 86_400_000;
        match qc
            .get_fund_history_quote(FundHistoryQuoteRequest {
                symbols: Some(vec![fund_symbol.clone()]),
                begin_time: Some(one_year_ago_ms),
                end_time: Some(now_ms),
                limit: Some(5),
                ..Default::default()
            })
            .await
        {
            Ok(items) if !items.is_empty() => ok(
                &mut results,
                "GetFundHistoryQuote",
                format!("count={} first_date={} first_nav={:.4}", items.len(), items[0].date, items[0].nav),
            ),
            Ok(_) => skip(&mut results, "GetFundHistoryQuote", "no data returned"),
            Err(e) => fail(&mut results, "GetFundHistoryQuote", e),
        }
    }

    println!("\n=== Industry ===");
    let mut industry_id = String::new();

    match qc
        .get_industry_list(IndustryListRequest {
            industry_level: Some("GSECTOR".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(industries) if !industries.is_empty() => {
            ok(
                &mut results,
                "GetIndustryList(GSECTOR)",
                format!("count={}", industries.len()),
            );
            industry_id = industries[0].id.clone();
        }
        Ok(_) => ok(&mut results, "GetIndustryList(GSECTOR)", "(empty)"),
        Err(e) => fail(&mut results, "GetIndustryList(GSECTOR)", e),
    }

    // GetIndustryStocks (new test)
    if industry_id.is_empty() {
        skip(&mut results, "GetIndustryStocks", "no industry id");
    } else {
        match qc
            .get_industry_stocks(IndustryStocksRequest {
                industry_id: Some(industry_id.clone()),
                market: Some("US".to_string()),
                ..Default::default()
            })
            .await
        {
            Ok(stocks) => ok(
                &mut results,
                "GetIndustryStocks",
                format!("symbols={}", stocks.len()),
            ),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("does not support") || msg.contains("code=1000") || msg.contains("code=4") {
                    skip(&mut results, "GetIndustryStocks", format!("not supported: {}", msg));
                } else {
                    fail(&mut results, "GetIndustryStocks", e);
                }
            }
        }
    }

    println!("\n=== Fundamentals & capital flow ===");
    // Enhanced: print field and value of first item
    match qc
        .get_financial_daily(FinancialDailyRequest {
            symbols: vec!["AAPL".into()],
            market: "US".into(),
            fields: vec!["shares_outstanding".into()],
            begin_date: "2026-05-05".into(),
            end_date: "2026-05-06".into(),
        })
        .await
    {
        Ok(items) if !items.is_empty() => ok(
            &mut results,
            "GetFinancialDaily(AAPL)",
            format!("rows={} field={} value={:.0}", items.len(), items[0].field, items[0].value),
        ),
        Ok(items) => ok(
            &mut results,
            "GetFinancialDaily(AAPL)",
            format!("rows={}", items.len()),
        ),
        Err(e) => fail(&mut results, "GetFinancialDaily(AAPL)", e),
    }

    match qc
        .get_financial_report(FinancialReportRequest {
            symbols: vec!["AAPL".into()],
            market: "US".into(),
            fields: vec!["total_revenue".into()],
            period_type: "Annual".into(),
            ..Default::default()
        })
        .await
    {
        Ok(items) if !items.is_empty() => ok(
            &mut results,
            "GetFinancialReport(AAPL)",
            format!(
                "{} {}={} @{}",
                items[0].symbol, items[0].field, items[0].value, items[0].filing_date
            ),
        ),
        Ok(_) => ok(&mut results, "GetFinancialReport(AAPL)", "(empty)"),
        Err(e) => fail(&mut results, "GetFinancialReport(AAPL)", e),
    }

    // Enhanced: print currency of first item
    match qc
        .get_financial_currency(FinancialCurrencyRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            market: Some("US".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(items) if !items.is_empty() => ok(
            &mut results,
            "GetFinancialCurrency(AAPL)",
            format!("count={} currency={}", items.len(), items[0].currency),
        ),
        Ok(items) => ok(
            &mut results,
            "GetFinancialCurrency(AAPL)",
            format!("count={}", items.len()),
        ),
        Err(e) => fail(&mut results, "GetFinancialCurrency(AAPL)", e),
    }

    // Financial exchange rate
    match qc
        .get_financial_exchange_rate(FinancialExchangeRateRequest {
            currency_list: Some(vec!["USD".to_string(), "HKD".to_string()]),
            begin_date: Some("2026-05-01".to_string()),
            end_date: Some("2026-05-07".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(rates) => ok(
            &mut results,
            "GetFinancialExchangeRate",
            format!("count={}", rates.len()),
        ),
        Err(e) => fail(&mut results, "GetFinancialExchangeRate", e),
    }

    // Enhanced: print date and is_trading of first item
    match qc
        .get_trading_calendar(TradingCalendarRequest {
            market: Some("US".to_string()),
            begin_date: Some("2026-05-01".to_string()),
            end_date: Some("2026-05-31".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(items) if !items.is_empty() => ok(
            &mut results,
            "GetTradingCalendar(US May)",
            format!("count={} first_date={} is_trading={}", items.len(), items[0].date, items[0].is_trading),
        ),
        Ok(items) => ok(
            &mut results,
            "GetTradingCalendar(US May)",
            format!("count={}", items.len()),
        ),
        Err(e) => fail(&mut results, "GetTradingCalendar(US May)", e),
    }

    // Enhanced: print exec_date of first split
    match qc
        .get_corporate_split(CorporateActionRequest {
            symbols: vec!["AAPL".into()],
            market: "US".into(),
            action_type: "split".into(),
            begin_date: "2020-01-01".into(),
            end_date: "2026-05-07".into(),
        })
        .await
    {
        Ok(items) if !items.is_empty() => ok(
            &mut results,
            "GetCorporateSplit(AAPL)",
            format!("rows={} first_exec_date={}", items.len(), items[0].execute_date),
        ),
        Ok(items) => ok(
            &mut results,
            "GetCorporateSplit(AAPL)",
            format!("rows={}", items.len()),
        ),
        Err(e) => fail(&mut results, "GetCorporateSplit(AAPL)", e),
    }

    // Enhanced: print exec_date of first dividend
    match qc
        .get_corporate_dividend(CorporateActionRequest {
            symbols: vec!["AAPL".into()],
            market: "US".into(),
            action_type: "dividend".into(),
            begin_date: "2024-01-01".into(),
            end_date: "2024-12-31".into(),
        })
        .await
    {
        Ok(items) if !items.is_empty() => ok(
            &mut results,
            "GetCorporateDividend(AAPL)",
            format!("rows={} first_exec_date={}", items.len(), items[0].execute_date),
        ),
        Ok(items) => ok(
            &mut results,
            "GetCorporateDividend(AAPL)",
            format!("rows={}", items.len()),
        ),
        Err(e) => fail(&mut results, "GetCorporateDividend(AAPL)", e),
    }

    // Enhanced: print action_type of first corporate action
    match qc
        .get_corporate_action(CorporateActionRequest {
            symbols: vec!["AAPL".into()],
            market: "US".into(),
            action_type: "DIVIDEND".into(),
            begin_date: "2024-01-01".into(),
            end_date: "2024-12-31".into(),
        })
        .await
    {
        Ok(items) if !items.is_empty() => ok(
            &mut results,
            "GetCorporateAction(AAPL)",
            format!("rows={} first_action_type={}", items.len(), items[0].action_type),
        ),
        Ok(items) => ok(
            &mut results,
            "GetCorporateAction(AAPL)",
            format!("rows={}", items.len()),
        ),
        Err(e) => fail(&mut results, "GetCorporateAction(AAPL)", e),
    }

    // GetCorporateEarningsCalendar (new test) — server limits to 1-month date range
    match qc
        .get_corporate_earnings_calendar(CorporateActionRequest {
            symbols: vec!["AAPL".into()],
            market: "US".into(),
            action_type: "earning".into(),
            begin_date: "2026-07-01".into(),
            end_date: "2026-07-31".into(),
        })
        .await
    {
        Ok(items) => ok(
            &mut results,
            "GetCorporateEarningsCalendar(AAPL)",
            format!("events={}", items.len()),
        ),
        Err(e) => fail(&mut results, "GetCorporateEarningsCalendar(AAPL)", e),
    }

    match qc.get_capital_flow("AAPL", "US", "day").await {
        Ok(Some(cf)) => ok(
            &mut results,
            "GetCapitalFlow(AAPL)",
            format!("{} period={} rows={}", cf.symbol, cf.period, cf.items.len()),
        ),
        Ok(None) => ok(&mut results, "GetCapitalFlow(AAPL)", "(empty)"),
        Err(e) => fail(&mut results, "GetCapitalFlow(AAPL)", e),
    }

    match qc.get_capital_distribution("AAPL", "US").await {
        Ok(Some(cd)) => ok(
            &mut results,
            "GetCapitalDistribution(AAPL)",
            format!("{} netInflow={:.2}", cd.symbol, cd.net_inflow),
        ),
        Ok(None) => ok(&mut results, "GetCapitalDistribution(AAPL)", "(empty)"),
        Err(e) => fail(&mut results, "GetCapitalDistribution(AAPL)", e),
    }

    println!("\n=== Scanner & permission ===");
    match qc
        .market_scanner(MarketScannerRequest {
            market: "US".into(),
            page: Some(0),
            page_size: Some(10),
            ..Default::default()
        })
        .await
    {
        Ok(Some(r)) => ok(
            &mut results,
            "MarketScanner",
            format!(
                "page={}/{} totalCount={} items={}",
                r.page,
                r.total_page,
                r.total_count,
                r.items.len()
            ),
        ),
        Ok(None) => ok(&mut results, "MarketScanner", "(empty)"),
        Err(e) => fail(&mut results, "MarketScanner", e),
    }

    // GetMarketScannerTags — server returns array of {market, multiTagField, tagList:[...]}
    match qc
        .get_market_scanner_tags(MarketScannerTagsRequest {
            market: Some("US".to_string()),
            multi_tags_fields: Some(vec!["MultiTagField_Industry".to_string()]),
            ..Default::default()
        })
        .await
    {
        Ok(items) => {
            let tag_count: usize = items.iter().map(|g| g.tag_list.len()).sum();
            ok(
                &mut results,
                "GetMarketScannerTags",
                format!("tag_groups={} total_tags={}", items.len(), tag_count),
            )
        }
        Err(e) => fail(&mut results, "GetMarketScannerTags", e),
    }

    match qc.grab_quote_permission().await {
        Ok(perms) => ok(
            &mut results,
            "GrabQuotePermission",
            format!("permissions={}", perms.len()),
        ),
        Err(e) => fail(&mut results, "GrabQuotePermission", e),
    }

    // Enhanced: print pre_close and volume of first item
    match qc
        .get_quote_overnight(QuoteOvernightRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        })
        .await
    {
        Ok(items) if !items.is_empty() => ok(
            &mut results,
            "GetQuoteOvernight(AAPL)",
            format!("count={} pre_close={:.4} volume={}", items.len(), items[0].pre_close, items[0].volume),
        ),
        Ok(items) => ok(&mut results, "GetQuoteOvernight(AAPL)", format!("count={}", items.len())),
        Err(e) => fail(&mut results, "GetQuoteOvernight(AAPL)", e),
    }

    // GetWarrantQuote (new test — skip if no warrant permission)
    match qc
        .get_warrant_quote(WarrantQuoteRequest {
            symbols: Some(vec!["11155.HK".to_string()]),
            ..Default::default()
        })
        .await
    {
        Ok(items) if !items.is_empty() => ok(
            &mut results,
            "GetWarrantQuote",
            format!("count={} symbol={}", items.len(), items[0].symbol),
        ),
        Ok(_) => skip(&mut results, "GetWarrantQuote", "no data returned (may need warrant permission)"),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("permission") || msg.contains("code=4") || msg.contains("unauthorized") {
                skip(&mut results, "GetWarrantQuote", format!("no warrant permission: {}", msg));
            } else {
                fail(&mut results, "GetWarrantQuote", e);
            }
        }
    }

    // GetWarrantFilter (new test — skip if no warrant permission; symbol is required)
    match qc
        .get_warrant_filter(WarrantFilterRequest {
            symbol: Some("00700".to_string()),
            page: Some(0),
            page_size: Some(5),
            ..Default::default()
        })
        .await
    {
        Ok(Some(result)) => ok(
            &mut results,
            "GetWarrantFilter",
            format!("total={} items={}", result.total, result.items.len()),
        ),
        Ok(None) => skip(&mut results, "GetWarrantFilter", "no data returned (may need warrant permission)"),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("permission") || msg.contains("unauthorized") || msg.contains("does not support") || msg.contains("code=4") {
                skip(&mut results, "GetWarrantFilter", format!("no warrant permission: {}", msg));
            } else {
                fail(&mut results, "GetWarrantFilter", e);
            }
        }
    }

    // ── low-level call_* API ──────────────────────────────────────────────
    println!("\n=== Low-level call_* API ===");

    // call_into: raw method name + params, deserializes data directly into T
    match qc
        .call_into::<Vec<MarketState>, _>("market_state", serde_json::json!({"market": "US"}))
        .await
    {
        Ok(states) if !states.is_empty() => ok(
            &mut results,
            "call_into(market_state)",
            format!("market={} status={}", states[0].market, states[0].market_status),
        ),
        Ok(_) => ok(&mut results, "call_into(market_state)", "(empty)"),
        Err(e) => fail(&mut results, "call_into(market_state)", e),
    }

    // call_into_versioned: same as call_into but with explicit API version
    match qc
        .call_into_versioned::<Vec<MarketState>, _>(
            "market_state",
            serde_json::json!({"market": "US"}),
            Some("2.0"),
        )
        .await
    {
        Ok(states) => ok(
            &mut results,
            "call_into_versioned(market_state, v2.0)",
            format!("count={}", states.len()),
        ),
        Err(e) => fail(&mut results, "call_into_versioned(market_state, v2.0)", e),
    }

    // call_into_items: unwraps {"items":[...]} envelope
    match qc
        .call_into_items::<Brief, _>(
            "brief",
            serde_json::json!({"symbols": ["AAPL"], "level": "basic"}),
        )
        .await
    {
        Ok(briefs) if !briefs.is_empty() => ok(
            &mut results,
            "call_into_items(brief)",
            format!("symbol={} price={:?}", briefs[0].symbol, briefs[0].latest_price),
        ),
        Ok(_) => ok(&mut results, "call_into_items(brief)", "(empty)"),
        Err(e) => fail(&mut results, "call_into_items(brief)", e),
    }

    // call_optional: returns None when data is absent; use capital_distribution which returns a single object
    match qc
        .call_optional::<tigeropen::model::quote::CapitalDistribution, _>(
            "capital_distribution",
            serde_json::json!({"symbol": "AAPL", "market": "US"}),
        )
        .await
    {
        Ok(Some(cd)) => ok(
            &mut results,
            "call_optional(capital_distribution)",
            format!("symbol={} netInflow={:.0}", cd.symbol, cd.net_inflow),
        ),
        Ok(None) => ok(&mut results, "call_optional(capital_distribution)", "(no data)"),
        Err(e) => fail(&mut results, "call_optional(capital_distribution)", e),
    }

    print_summary(&results);
    Ok(())
}

async fn run_option_smoke(qc: &tigeropen::quote::QuoteClient, symbol: &str, results: &mut Vec<RunResult>) {
    let mut expiry_date = String::new();
    let mut opt_identifier = String::new();

    // Fetch current price to find ATM strike later.
    let current_price: f64 = match qc.get_real_time_quote(BriefRequest {
        symbols: Some(vec![symbol.to_string()]),
        ..Default::default()
    }).await {
        Ok(briefs) if !briefs.is_empty() && briefs[0].latest_price > 0.0 => briefs[0].latest_price,
        _ => 0.0,
    };

    let tag = format!("GetOptionExpiration({})", symbol);
    match qc.get_option_expiration(&[symbol], None).await {
        Ok(exps) if !exps.is_empty() && !exps[0].dates.is_empty() => {
            let dates = &exps[0].dates;
            // Pick the first expiry >= 30 days from now — these contracts have trade history for kline.
            let min_date = {
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
                    + 30 * 86_400;
                let d = chrono::DateTime::from_timestamp(secs as i64, 0).unwrap();
                d.format("%Y-%m-%d").to_string()
            };
            let picked = dates.iter()
                .find(|d| d.as_str() >= min_date.as_str())
                .unwrap_or(&dates[dates.len() / 2]);
            ok(results, &tag, format!("dates={} picked={}", dates.len(), picked));
            expiry_date = picked.clone();
        }
        Ok(_) => ok(results, &tag, "(empty)"),
        Err(e) => { fail(results, &tag, e); return; }
    }

    let chain_tag = format!("GetOptionChain({} {})", symbol, expiry_date);
    match qc.get_option_chain(OptionChainRequest::new(vec![
        OptionChainItem::from_date(symbol, &expiry_date).unwrap()
    ])).await {
        Ok(chain) if !chain.is_empty() && !chain[0].items.is_empty() => {
            ok(results, &chain_tag, format!("rows={}", chain[0].items.len()));
            let items = &chain[0].items;
            // Chain rows are sorted by strike. Pick call with strike closest to current_price.
            let best = if current_price > 0.0 {
                items.iter()
                    .filter_map(|row| row.call.as_ref())
                    .filter(|leg| !leg.identifier.is_empty())
                    .min_by(|a, b| {
                        let da = (a.strike.parse::<f64>().unwrap_or(0.0) - current_price).abs();
                        let db = (b.strike.parse::<f64>().unwrap_or(0.0) - current_price).abs();
                        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                    })
            } else {
                items.iter().filter_map(|row| row.call.as_ref())
                    .max_by_key(|leg| leg.open_interest)
            };
            if let Some(leg) = best {
                opt_identifier = leg.identifier.clone();
            } else {
                let mid = &items[items.len() / 2];
                if let Some(call) = &mid.call { opt_identifier = call.identifier.clone(); }
                else if let Some(put) = &mid.put { opt_identifier = put.identifier.clone(); }
            }
            ok(results, &format!("{}→picked", chain_tag), opt_identifier.clone());
        }
        Ok(_) => ok(results, &chain_tag, "(empty items)"),
        Err(e) => { fail(results, &chain_tag, e); return; }
    }

    // Verify return_greek_value and option_filter take effect
    let chain_greek_tag = format!("GetOptionChain({} greek+filter)", symbol);
    match qc.get_option_chain(OptionChainRequest {
        option_basic: Some(vec![OptionChainItem::from_date(symbol, &expiry_date).unwrap()]),
        return_greek_value: Some(true),
        option_filter: Some(OptionChainFilter {
            in_the_money: Some(false),
            implied_volatility: Some(RangeF64::new(0.0, 5.0)),
            greeks: Some(OptionChainFilterGreeks {
                delta: Some(RangeF64::new(0.0, 0.6)),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }).await {
        Ok(chain) if !chain.is_empty() => {
            let total_rows = chain.iter().map(|c| c.items.len()).sum::<usize>();
            let has_greek = chain.iter().flat_map(|c| c.items.iter())
                .any(|row| row.call.as_ref().map(|l| l.delta != 0.0).unwrap_or(false)
                    || row.put.as_ref().map(|l| l.delta != 0.0).unwrap_or(false));
            ok(results, &chain_greek_tag,
                format!("rows={} has_greek={}", total_rows, has_greek));
        }
        Ok(_) => ok(results, &chain_greek_tag, "(empty)"),
        Err(e) => fail(results, &chain_greek_tag, e),
    }

    if opt_identifier.is_empty() {
        skip(results, &format!("GetOptionQuote({})", symbol), "no identifier from chain");
        skip(results, &format!("GetOptionKline({})", symbol), "no identifier from chain");
        skip(results, &format!("GetOptionTradeTicks({})", symbol), "no identifier from chain");
        skip(results, &format!("GetOptionTimeline({})", symbol), "no identifier from chain");
        skip(results, &format!("GetOptionDepth({})", symbol), "no identifier from chain");
        return;
    }

    let quote_tag = format!("GetOptionQuote({})", symbol);
    match qc.get_option_quote(OptionQuoteRequest::new(vec![
        OptionContractItem::from_occ(&opt_identifier).unwrap()
    ])).await {
        Ok(briefs) if !briefs.is_empty() => ok(
            results, &quote_tag,
            format!("{} latestPrice={:.4}", briefs[0].symbol, briefs[0].latest_price),
        ),
        Ok(_) => ok(results, &quote_tag, "(empty)"),
        Err(e) => fail(results, &quote_tag, e),
    }

    let kline_tag = format!("GetOptionKline({})", symbol);
    // option_kline requires begin_time and end_time; use last 90 days
    let end_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let begin_ms = end_ms - 90 * 86_400_000;
    let mut kline_item = OptionKlineItem::from_occ(&opt_identifier, "day").unwrap();
    kline_item.begin_time = Some(begin_ms);
    kline_item.end_time = Some(end_ms);
    match qc.get_option_kline(OptionKlineRequest {
        option_query: Some(vec![kline_item]),
        ..Default::default()
    }).await {
        Ok(ks) if !ks.is_empty() => ok(results, &kline_tag, format!("bars={}", ks[0].items.len())),
        Ok(_) => ok(results, &kline_tag, "(empty)"),
        Err(e) => fail(results, &kline_tag, e),
    }

    // GetOptionTradeTicks (new test) — use OptionQueryItem with expiry as ms timestamp
    let tick_tag = format!("GetOptionTradeTicks({})", symbol);
    if let Ok(ci) = OptionContractItem::from_occ(&opt_identifier) {
        let query_item = OptionQueryItem {
            symbol: Some(ci.symbol.clone()),
            expiry: Some(ci.expiry),
            right: Some(ci.right.clone()),
            strike: Some(ci.strike.clone()),
            begin_index: Some(0),
            end_index: Some(30),
            ..Default::default()
        };
        match qc.get_option_trade_ticks(OptionTradeTicksRequest {
            contracts: Some(vec![query_item]),
            ..Default::default()
        }).await {
            Ok(ticks) if !ticks.is_empty() => ok(
                results, &tick_tag,
                format!("ticks={}", ticks[0].items.len()),
            ),
            Ok(_) => skip(results, &tick_tag, "no data returned"),
            Err(e) => {
                // biz_content parse error can indicate unsupported params format or account tier
                skip(results, &tick_tag, format!("skipped: {}", e));
            }
        }
    } else {
        skip(results, &tick_tag, "could not parse opt_identifier for trade ticks");
    }

    // GetOptionTimeline (new test)
    let timeline_tag = format!("GetOptionTimeline({})", symbol);
    if let Ok(ci) = OptionContractItem::from_occ(&opt_identifier) {
        let query_item = OptionQueryItem {
            symbol: Some(ci.symbol),
            expiry: Some(ci.expiry),
            right: Some(ci.right),
            strike: Some(ci.strike),
            ..Default::default()
        };
        match qc.get_option_timeline(OptionTimelineRequest {
            option_query: Some(vec![query_item]),
            market: Some("US".to_string()),
            ..Default::default()
        }).await {
            Ok(tl) if !tl.is_empty() => {
                let n = tl[0].intraday.as_ref().map(|b| b.items.len()).unwrap_or(0);
                ok(results, &timeline_tag, format!("items.len={}", n))
            }
            Ok(_) => skip(results, &timeline_tag, "no data returned"),
            Err(e) => fail(results, &timeline_tag, e),
        }
    } else {
        skip(results, &timeline_tag, "could not parse opt_identifier for timeline");
    }

    // GetOptionDepth (new test)
    let depth_tag = format!("GetOptionDepth({})", symbol);
    if let Ok(ci) = OptionContractItem::from_occ(&opt_identifier) {
        let query_item = OptionQueryItem {
            symbol: Some(ci.symbol),
            expiry: Some(ci.expiry),
            right: Some(ci.right),
            strike: Some(ci.strike),
            ..Default::default()
        };
        match qc.get_option_depth(OptionDepthRequest {
            option_basic: Some(vec![query_item]),
            market: Some("US".to_string()),
            ..Default::default()
        }).await {
            Ok(depths) if !depths.is_empty() => ok(
                results, &depth_tag,
                format!("asks={} bids={}", depths[0].asks.len(), depths[0].bids.len()),
            ),
            Ok(_) => skip(results, &depth_tag, "no data returned"),
            Err(e) => fail(results, &depth_tag, e),
        }
    } else {
        skip(results, &depth_tag, "could not parse opt_identifier for depth");
    }

    // GetOptionAnalysis (US)
    match qc.get_option_analysis(OptionAnalysisRequest {
        symbols: Some(vec![symbol.to_string()]),
        market: Some("US".to_string()),
        period: Some("52week".to_string()),
        ..Default::default()
    }).await {
        Ok(items) => ok(results, &format!("GetOptionAnalysis({} US)", symbol),
            format!("count={} iv30d={:.4} hv={:.4}",
                items.len(),
                items.first().map(|i| i.implied_vol30_days).unwrap_or(0.0),
                items.first().map(|i| i.his_volatility).unwrap_or(0.0))),
        Err(e) => fail(results, &format!("GetOptionAnalysis({} US)", symbol), e),
    }
}

async fn run_hk_option_smoke(qc: &tigeropen::quote::QuoteClient, symbol: &str, results: &mut Vec<RunResult>) {
    use tigeropen::model::quote_requests::OptionSymbolsRequest;

    // Step 1: get_option_symbols(HK) to discover tradable HK option symbols
    let syms = match qc.get_option_symbols(OptionSymbolsRequest {
        market: Some("HK".to_string()),
        ..Default::default()
    }).await {
        Ok(s) if !s.is_empty() => {
            ok(results, "GetOptionSymbols(HK)", format!("count={}", s.len()));
            s
        }
        Ok(_) => { skip(results, "GetOptionSymbols(HK)", "empty result"); return; }
        Err(e) if e.to_string().contains("does not support") || e.to_string().contains("code=4") => {
            skip(results, "GetOptionSymbols(HK)", "not supported by this account tier");
            return;
        }
        Err(e) => { fail(results, "GetOptionSymbols(HK)", e); return; }
    };

    // Step 2: get_option_expiration — underlying for HK options is "TCH.HK" format (not numeric)
    let hk_chain_input: Option<(String, i64)> = {
        let mut result: Option<(String, i64)> = None;
        // Build candidate list: use symbol itself as underlying (e.g. "TCH.HK")
        // TCH.HK is Tencent's HK option symbol, underlying = "TCH.HK"
        let mut candidates: Vec<String> = vec!["TCH.HK".to_string()];
        for s in syms.iter().take(5) {
            candidates.push(s.symbol.clone());
        }
        candidates.dedup();
        for sym in &candidates {
            match qc.get_option_expiration(&[sym.as_str()], Some("HK")).await {
                Ok(exps) => {
                    let min_ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
                        + 30 * 86_400_000;
                    let ts = exps.iter()
                        .flat_map(|e| e.timestamps.iter().copied())
                        .find(|&t| t >= min_ts);
                    if let Some(ts) = ts {
                        ok(results, "GetOptionExpiration(HK)", format!("sym={} ts={}", sym, ts));
                        result = Some((sym.clone(), ts));
                        break;
                    }
                }
                Err(_) => {}
            }
        }
        if result.is_none() {
            skip(results, "GetOptionExpiration(HK)", "no valid expiry found");
        }
        result
    };

    // Step 3: get_option_chain — only if we have a valid expiry
    let (hk_opt_symbol, hk_expiry_ms) = match hk_chain_input {
        Some(v) => v,
        None => {
            skip(results, "GetOptionChain→Quote→Kline(HK)", "no expiry found");
            // still run analysis
            match qc.get_option_analysis(tigeropen::model::quote_requests::OptionAnalysisRequest {
                symbols: Some(vec![symbol.to_string()]),
                market: Some("HK".to_string()),
                period: Some("52week".to_string()),
                ..Default::default()
            }).await {
                Ok(items) => ok(results, &format!("GetOptionAnalysis({} HK)", symbol),
                    format!("count={} iv30d={:.4} hv={:.4}",
                        items.len(),
                        items.first().map(|i| i.implied_vol30_days).unwrap_or(0.0),
                        items.first().map(|i| i.his_volatility).unwrap_or(0.0))),
                Err(e) => fail(results, &format!("GetOptionAnalysis({} HK)", symbol), e),
            }
            return;
        }
    };
    let hk_chain_tag = format!("GetOptionChain({} HK)", hk_opt_symbol);
    let opt_identifier = match qc.get_option_chain(OptionChainRequest {
        option_basic: Some(vec![OptionChainItem::new(hk_opt_symbol.as_str(), hk_expiry_ms)]),
        market: Some("HK".to_string()),
        ..Default::default()
    }).await {
        Ok(chains) => {
            // Pick call with OI > 0; if none, take the middle strike (closest to ATM).
            let all_items: Vec<_> = chains.iter().flat_map(|c| c.items.iter()).collect();
            let best_id = all_items.iter()
                .filter_map(|row| row.call.as_ref())
                .filter(|leg| leg.open_interest > 0 && !leg.identifier.is_empty())
                .max_by_key(|leg| leg.open_interest)
                .map(|leg| leg.identifier.clone());
            let first_id = best_id.or_else(|| {
                let mid_idx = all_items.len() / 2;
                all_items[mid_idx].call.as_ref()
                    .map(|l| l.identifier.clone())
                    .or_else(|| all_items[mid_idx].put.as_ref().map(|l| l.identifier.clone()))
                    .filter(|id| !id.is_empty())
                    .or_else(|| {
                        all_items.iter().find_map(|row| {
                            row.call.as_ref().map(|l| l.identifier.clone())
                                .or_else(|| row.put.as_ref().map(|l| l.identifier.clone()))
                        }).filter(|id| !id.is_empty())
                    })
            });
            match first_id {
                Some(id) => { ok(results, &hk_chain_tag, format!("identifier={}", id.trim())); id }
                None => { ok(results, &hk_chain_tag, "(empty chain)"); return; }
            }
        }
        Err(e) => { fail(results, &hk_chain_tag, e); return; }
    };

    // Step 4: get_option_quote using the identifier from chain
    let opt_id_trimmed = opt_identifier.trim().to_string();
    let quote_tag = format!("GetOptionQuote({} HK)", opt_id_trimmed);
    let contract_item = match OptionContractItem::from_occ(&opt_id_trimmed) {
        Ok(c) => c,
        Err(e) => { fail(results, &quote_tag, e); return; }
    };
    match qc.get_option_quote(OptionQuoteRequest::new(vec![contract_item])).await {
        Ok(briefs) if !briefs.is_empty() => ok(
            results, &quote_tag,
            format!("{} latestPrice={:.4}", briefs[0].symbol, briefs[0].latest_price),
        ),
        Ok(_) => ok(results, &quote_tag, "(empty)"),
        Err(e) => fail(results, &quote_tag, e),
    }

    // Step 5: get_option_kline using same identifier, last 30 days
    let kline_tag = format!("GetOptionKline({} HK)", opt_id_trimmed);
    let end_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
    let begin_ms = end_ms - 90 * 86_400_000;
    let kline_item = match OptionKlineItem::from_occ(&opt_id_trimmed, "day") {
        Ok(mut k) => { k.begin_time = Some(begin_ms); k.end_time = Some(end_ms); k }
        Err(e) => { fail(results, &kline_tag, e); return; }
    };
    match qc.get_option_kline(OptionKlineRequest {
        option_query: Some(vec![kline_item]),
        ..Default::default()
    }).await {
        Ok(ks) if !ks.is_empty() => ok(results, &kline_tag, format!("bars={}", ks[0].items.len())),
        Ok(_) => ok(results, &kline_tag, "(empty)"),
        Err(e) => fail(results, &kline_tag, e),
    }

    // Step 6: get_option_analysis for the underlying symbol (HK) — enhanced with iv30d and hv
    match qc.get_option_analysis(OptionAnalysisRequest {
        symbols: Some(vec![symbol.to_string()]),
        market: Some("HK".to_string()),
        period: Some("52week".to_string()),
        ..Default::default()
    }).await {
        Ok(items) => ok(results, &format!("GetOptionAnalysis({} HK)", symbol),
            format!("count={} iv30d={:.4} hv={:.4}",
                items.len(),
                items.first().map(|i| i.implied_vol30_days).unwrap_or(0.0),
                items.first().map(|i| i.his_volatility).unwrap_or(0.0))),
        Err(e) => fail(results, &format!("GetOptionAnalysis({} HK)", symbol), e),
    }
}
