//! Quote example - covers every QuoteClient public method.
//!
//! Config is auto-discovered from:
//!   1. ./tiger_openapi_config.properties
//!   2. ~/.tigeropen/tiger_openapi_config.properties
//!
//! Individual failures do not abort subsequent calls; a final PASS/FAIL/SKIP summary is printed.
//!
//! Run: `cargo run --example quote_example`

use tigeropen::client::http_client::HttpClient;
use tigeropen::config::ClientConfig;
use tigeropen::model::quote::{
    CorporateActionRequest, FinancialDailyRequest, FinancialReportRequest, FutureKlineRequest,
    MarketScannerRequest,
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
    println!("[ OK ] {:<36} {}", name, truncate(&note, 140));
    results.push(RunResult { name: name.into(), outcome: Outcome::Pass, detail: note });
}

fn fail(results: &mut Vec<RunResult>, name: &str, err: impl std::fmt::Display) {
    let detail = format!("{}", err);
    println!("[FAIL] {:<36} {}", name, detail);
    results.push(RunResult { name: name.into(), outcome: Outcome::Fail, detail });
}

fn skip(results: &mut Vec<RunResult>, name: &str, reason: impl Into<String>) {
    let reason = reason.into();
    println!("[SKIP] {:<36} {}", name, reason);
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
    let config = ClientConfig::builder().build()?;
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

    match qc.get_brief(&["AAPL", "TSLA"]).await {
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

    match qc.get_trade_tick(&["AAPL"]).await {
        Ok(tt) if !tt.is_empty() => ok(
            &mut results,
            "GetTradeTick",
            format!("ticks={}", tt[0].items.len()),
        ),
        Ok(_) => ok(&mut results, "GetTradeTick", "(empty)"),
        Err(e) => fail(&mut results, "GetTradeTick", e),
    }

    match qc.get_quote_depth("AAPL", "US").await {
        Ok(d) if !d.is_empty() => ok(
            &mut results,
            "GetQuoteDepth(AAPL)",
            format!("asks={} bids={}", d[0].asks.len(), d[0].bids.len()),
        ),
        Ok(_) => ok(&mut results, "GetQuoteDepth(AAPL)", "(empty)"),
        Err(e) => fail(&mut results, "GetQuoteDepth(AAPL)", e),
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
    } else {
        match qc.get_future_real_time_quote(&[contract_code.as_str()]).await {
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

    print_summary(&results);
    Ok(())
}
