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

use tigeropen::client::http_client::HttpClient;
use tigeropen::config::ClientConfig;
use tigeropen::model::quote::{
    CorporateActionRequest, FinancialDailyRequest, FinancialReportRequest, FutureKlineRequest,
    MarketScannerRequest,
};
use tigeropen::model::quote_requests::{
    AllFutureContractsRequest, BarsRequest, BriefRequest, DepthQuoteRequest,
    FinancialCurrencyRequest, FinancialExchangeRateRequest, FutureBarsRequest, FutureBriefRequest,
    FutureContractSingleRequest, FutureDepthRequest, FutureTradingTimesRequest,
    FundSymbolsRequest, IndustryListRequest, KlineQuotaRequest,
    QuoteOvernightRequest, QuotePermissionRequest, StockDetailsRequest, StockIndustryRequest,
    TimelineHistoryRequest, TradeMetasRequest, TradeRankRequest, TradeTickRequest,
    TradingCalendarRequest,
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

    let http = HttpClient::with_quote_server(config);
    let qc = QuoteClient::new(&http);

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
        .get_brief(BriefRequest {
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
            ok(&mut results, "GetBrief", s.join(" "));
        }
        Err(e) => fail(&mut results, "GetBrief", e),
    }

    match qc.get_kline("AAPL", "day").await {
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

    // v0.4.0 new signature: TradeTickRequest
    match qc
        .get_trade_tick(TradeTickRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        })
        .await
    {
        Ok(tt) if !tt.is_empty() => ok(
            &mut results,
            "GetTradeTick",
            format!("ticks={}", tt[0].items.len()),
        ),
        Ok(_) => ok(&mut results, "GetTradeTick", "(empty)"),
        Err(e) => fail(&mut results, "GetTradeTick", e),
    }

    // v0.4.0 new signature: DepthQuoteRequest
    match qc
        .get_quote_depth(DepthQuoteRequest {
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
        .get_symbols(tigeropen::model::quote_requests::SymbolsRequest {
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
        .get_symbol_names(tigeropen::model::quote_requests::SymbolsRequest {
            market: Some("US".to_string()),
            sec_type: Some("STK".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(names) => ok(&mut results, "GetSymbolNames(US STK)", format!("count={}", names.len())),
        Err(e) => fail(&mut results, "GetSymbolNames(US STK)", e),
    }

    match qc
        .get_trade_metas(TradeMetasRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        })
        .await
    {
        Ok(metas) => ok(&mut results, "GetTradeMetas(AAPL)", format!("count={}", metas.len())),
        Err(e) => fail(&mut results, "GetTradeMetas(AAPL)", e),
    }

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
            format!("symbol={}", details[0].symbol),
        ),
        Ok(_) => ok(&mut results, "GetStockDetails(AAPL)", "(empty)"),
        Err(e) => fail(&mut results, "GetStockDetails(AAPL)", e),
    }

    match qc
        .get_stock_delay_briefs(tigeropen::model::quote_requests::StockDelayBriefsRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        })
        .await
    {
        Ok(briefs) => ok(
            &mut results,
            "GetStockDelayBriefs(AAPL)",
            format!("count={}", briefs.len()),
        ),
        Err(e) => fail(&mut results, "GetStockDelayBriefs(AAPL)", e),
    }

    match qc
        .get_bars(BarsRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            period: Some("day".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .await
    {
        Ok(ks) if !ks.is_empty() => ok(
            &mut results,
            "GetBars(AAPL day)",
            format!("bars={}", ks[0].items.len()),
        ),
        Ok(_) => ok(&mut results, "GetBars(AAPL day)", "(empty)"),
        Err(e) => fail(&mut results, "GetBars(AAPL day)", e),
    }

    match qc
        .get_timeline_history(TimelineHistoryRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            date: Some("2026-05-06".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(tl) if !tl.is_empty() => ok(
            &mut results,
            "GetTimelineHistory(AAPL)",
            format!("count={}", tl.len()),
        ),
        Ok(_) => ok(&mut results, "GetTimelineHistory(AAPL)", "(empty)"),
        Err(e) => fail(&mut results, "GetTimelineHistory(AAPL)", e),
    }

    match qc
        .get_trade_rank(TradeRankRequest {
            market: Some("US".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(items) => ok(&mut results, "GetTradeRank(US)", format!("count={}", items.len())),
        Err(e) => fail(&mut results, "GetTradeRank(US)", e),
    }

    match qc
        .get_stock_industry(StockIndustryRequest {
            symbol: Some("AAPL".to_string()),
            market: Some("US".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(industries) => ok(
            &mut results,
            "GetStockIndustry(AAPL)",
            format!("count={}", industries.len()),
        ),
        Err(e) => fail(&mut results, "GetStockIndustry(AAPL)", e),
    }

    match qc
        .get_quote_permission(QuotePermissionRequest {
            ..Default::default()
        })
        .await
    {
        Ok(perms) => ok(&mut results, "GetQuotePermission", format!("count={}", perms.len())),
        Err(e) => fail(&mut results, "GetQuotePermission", e),
    }

    match qc
        .get_kline_quota(KlineQuotaRequest {
            with_details: Some(false),
            ..Default::default()
        })
        .await
    {
        Ok(quotas) => ok(&mut results, "GetKlineQuota", format!("count={}", quotas.len())),
        Err(e) => fail(&mut results, "GetKlineQuota", e),
    }

    println!("\n=== Options ===");
    let mut expiry_date = String::new();
    let mut opt_identifier = String::new();

    match qc.get_option_expiration("AAPL").await {
        Ok(exps) if !exps.is_empty() && !exps[0].dates.is_empty() => {
            ok(
                &mut results,
                "GetOptionExpiration(AAPL)",
                format!("dates={} first={}", exps[0].dates.len(), exps[0].dates[0]),
            );
            let dates = &exps[0].dates;
            expiry_date = dates[dates.len() / 2].clone();
        }
        Ok(_) => ok(&mut results, "GetOptionExpiration(AAPL)", "(empty)"),
        Err(e) => fail(&mut results, "GetOptionExpiration(AAPL)", e),
    }

    if expiry_date.is_empty() {
        skip(&mut results, "GetOptionChain", "no expiry available");
        skip(&mut results, "GetOptionBrief", "no expiry available");
        skip(&mut results, "GetOptionKline", "no expiry available");
    } else {
        match qc.get_option_chain("AAPL", &expiry_date).await {
            Ok(chain) if !chain.is_empty() && !chain[0].items.is_empty() => {
                ok(
                    &mut results,
                    &format!("GetOptionChain({})", expiry_date),
                    format!("rows={}", chain[0].items.len()),
                );
                let mid = &chain[0].items[chain[0].items.len() / 2];
                if let Some(call) = &mid.call {
                    opt_identifier = call.identifier.clone();
                } else if let Some(put) = &mid.put {
                    opt_identifier = put.identifier.clone();
                }
            }
            Ok(_) => ok(
                &mut results,
                &format!("GetOptionChain({})", expiry_date),
                "(empty items)",
            ),
            Err(e) => fail(&mut results, &format!("GetOptionChain({})", expiry_date), e),
        }

        if opt_identifier.is_empty() {
            skip(&mut results, "GetOptionBrief", "no identifier from chain");
            skip(&mut results, "GetOptionKline", "no identifier from chain");
        } else {
            match qc.get_option_brief(&[opt_identifier.as_str()]).await {
                Ok(briefs) if !briefs.is_empty() => ok(
                    &mut results,
                    "GetOptionBrief",
                    format!("{} latestPrice={:.4}", briefs[0].symbol, briefs[0].latest_price),
                ),
                Ok(_) => ok(&mut results, "GetOptionBrief", "(empty)"),
                Err(e) => fail(&mut results, "GetOptionBrief", e),
            }

            match qc.get_option_kline(&opt_identifier, "day").await {
                Ok(ks) if !ks.is_empty() => ok(
                    &mut results,
                    "GetOptionKline",
                    format!("bars={}", ks[0].items.len()),
                ),
                Ok(_) => ok(&mut results, "GetOptionKline", "(empty)"),
                Err(e) => fail(&mut results, "GetOptionKline", e),
            }
        }
    }

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
    } else {
        // v0.4.0 new signature: FutureBriefRequest
        match qc
            .get_future_real_time_quote(FutureBriefRequest {
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
                contract_codes: vec![contract_code.clone()],
                period: "day".into(),
                begin_time: -1,
                end_time: -1,
                limit: None,
                page_token: None,
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

        // GetFutureBars
        match qc
            .get_future_bars(FutureBarsRequest {
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
                &format!("GetFutureBars({})", contract_code),
                format!("bars={}", ks[0].items.len()),
            ),
            Ok(_) => ok(&mut results, &format!("GetFutureBars({})", contract_code), "(empty)"),
            Err(e) => fail(&mut results, &format!("GetFutureBars({})", contract_code), e),
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
    }

    println!("\n=== v0.4.0 期货扩展 smoke ===");
    // (contract_code already set above; skip if empty was already handled)

    println!("\n=== Fund ===");
    match qc.get_fund_symbols(FundSymbolsRequest::default()).await {
        Ok(syms) => ok(&mut results, "GetFundSymbols", format!("count={}", syms.len())),
        Err(e) => fail(&mut results, "GetFundSymbols", e),
    }

    println!("\n=== Industry ===");
    match qc
        .get_industry_list(IndustryListRequest {
            industry_level: Some("GSECTOR".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(industries) => ok(
            &mut results,
            "GetIndustryList(GSECTOR)",
            format!("count={}", industries.len()),
        ),
        Err(e) => fail(&mut results, "GetIndustryList(GSECTOR)", e),
    }

    println!("\n=== Fundamentals & capital flow ===");
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

    // Financial currency
    match qc
        .get_financial_currency(FinancialCurrencyRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            market: Some("US".to_string()),
            ..Default::default()
        })
        .await
    {
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

    // Trading calendar
    match qc
        .get_trading_calendar(TradingCalendarRequest {
            market: Some("US".to_string()),
            begin_date: Some("2026-05-01".to_string()),
            end_date: Some("2026-05-31".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(items) => ok(
            &mut results,
            "GetTradingCalendar(US May)",
            format!("count={}", items.len()),
        ),
        Err(e) => fail(&mut results, "GetTradingCalendar(US May)", e),
    }

    // Corporate split
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
        Ok(items) => ok(
            &mut results,
            "GetCorporateSplit(AAPL)",
            format!("rows={}", items.len()),
        ),
        Err(e) => fail(&mut results, "GetCorporateSplit(AAPL)", e),
    }

    // Corporate dividend
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
        Ok(items) => ok(
            &mut results,
            "GetCorporateDividend(AAPL)",
            format!("rows={}", items.len()),
        ),
        Err(e) => fail(&mut results, "GetCorporateDividend(AAPL)", e),
    }

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
        Ok(items) => ok(
            &mut results,
            "GetCorporateAction(AAPL)",
            format!("rows={}", items.len()),
        ),
        Err(e) => fail(&mut results, "GetCorporateAction(AAPL)", e),
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

    match qc.grab_quote_permission().await {
        Ok(perms) => ok(
            &mut results,
            "GrabQuotePermission",
            format!("permissions={}", perms.len()),
        ),
        Err(e) => fail(&mut results, "GrabQuotePermission", e),
    }

    // GetQuoteOvernight
    match qc
        .get_quote_overnight(QuoteOvernightRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        })
        .await
    {
        Ok(items) => ok(&mut results, "GetQuoteOvernight(AAPL)", format!("count={}", items.len())),
        Err(e) => fail(&mut results, "GetQuoteOvernight(AAPL)", e),
    }

    print_summary(&results);
    Ok(())
}
