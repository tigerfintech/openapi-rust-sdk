//! Integration test support helpers.
//!
//! Tests that use this module are gated by `TIGER_RUN_INTEG=true` so they do
//! not run during normal `cargo test` (unit stage). Set the env var and supply
//! credentials to run against the real gateway.
//!
//! Credential resolution order (matches Java / Python / Go SDKs):
//!   1. TIGEROPEN_TIGER_ID + TIGEROPEN_PRIVATE_KEY env vars (+ optionally TIGEROPEN_ACCOUNT)
//!   2. TIGEROPEN_PROPS_PATH — path to a .properties file

pub const ENV_RUN_INTEG: &str = "TIGER_RUN_INTEG";
pub const ENV_TIGER_ID: &str = "TIGEROPEN_TIGER_ID";
pub const ENV_PRIVATE_KEY: &str = "TIGEROPEN_PRIVATE_KEY";
pub const ENV_ACCOUNT: &str = "TIGEROPEN_ACCOUNT";
pub const ENV_SERVER_URL: &str = "TIGEROPEN_SERVER_URL";
pub const ENV_PROPS_PATH: &str = "TIGEROPEN_PROPS_PATH";

pub const DEFAULT_SERVER_URL: &str = "https://openapi.tigerfintech.com/gateway";
const MAX_DIAGNOSTIC_BYTES: usize = 3000;

/// Compact request/response context for integration-test failure messages.
pub fn diagnostic<Req: std::fmt::Debug, Resp: std::fmt::Debug>(req: &Req, resp: &Resp) -> String {
    let mut out = format!("request={req:?} response={resp:?}");
    if out.len() > MAX_DIAGNOSTIC_BYTES {
        let mut end = MAX_DIAGNOSTIC_BYTES;
        while !out.is_char_boundary(end) {
            end -= 1;
        }
        out.truncate(end);
        out.push_str("...<truncated>");
    }
    out
}

/// Returns true when integration tests should run.
pub fn is_integ_run() -> bool {
    std::env::var(ENV_RUN_INTEG).as_deref() == Ok("true")
}

/// Build a ClientConfig from env vars or a properties file.
///
/// Credential resolution order (matches Java / Python / Go SDKs):
///   1. TIGEROPEN_TIGER_ID + TIGEROPEN_PRIVATE_KEY env vars (+ optionally TIGEROPEN_ACCOUNT)
///   2. TIGEROPEN_PROPS_PATH — path to a .properties file, loaded via the SDK's own
///      `properties_file()` builder method so key resolution (including the
///      `private_key` > `private_key_pk8` > `private_key_pk1` priority) matches
///      production behavior instead of being re-implemented here.
///
/// Panics with a descriptive message when credentials are missing — a
/// misconfigured env is a real error, not a skip.
pub fn integ_config() -> tigeropen::config::ClientConfig {
    use std::time::Duration;

    let server_url =
        std::env::var(ENV_SERVER_URL).unwrap_or_else(|_| DEFAULT_SERVER_URL.to_string());

    let mut builder = tigeropen::config::ClientConfig::builder()
        .server_url(server_url.clone())
        .quote_server_url(server_url)
        .timeout(Duration::from_secs(30))
        .enable_dynamic_domain(false);

    match (std::env::var(ENV_TIGER_ID), std::env::var(ENV_PRIVATE_KEY)) {
        (Ok(tid), Ok(pk)) => {
            builder = builder.tiger_id(tid).private_key(pk);
            if let Ok(account) = std::env::var(ENV_ACCOUNT) {
                builder = builder.account(account);
            }
        }
        _ => {
            let path = std::env::var(ENV_PROPS_PATH)
                .expect("TIGEROPEN_PROPS_PATH env var required when env vars are absent");
            builder = builder.properties_file(&path);
        }
    }

    builder
        .build()
        .expect("Failed to build ClientConfig for integration tests")
}

/// Resolve a live US FOP (future option) contract on `CL` (crude oil) for the
/// nearest monthly expiry (3rd Friday of next month), mirroring Python's
/// `_get_option_contract_id` / TS's `resolveUsFopContract`. Returns `None` on
/// any gateway failure or empty response — the caller decides whether that
/// should fail or skip.
pub async fn resolve_us_fop_contract(
    tc: &tigeropen::trade::TradeClient,
) -> Option<tigeropen::model::Contract> {
    use tigeropen::model::trade_requests::DerivativeContractsRequest;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let now_days = now_ms / 86_400_000;
    let (y, m, _) = civil_from_days(now_days);
    let (next_y, next_m) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    let first_day_days = days_from_civil(next_y, next_m, 1);
    let first_day_weekday = ((first_day_days % 7) + 11) % 7; // 0=Mon..6=Sun (1970-01-01 was Thu)
    let days_to_first_friday = (4 - first_day_weekday + 7) % 7;
    let third_friday_days = first_day_days + days_to_first_friday + 14;
    let (fy, fm, fd) = civil_from_days(third_friday_days);
    let expiry = format!("{:04}{:02}{:02}", fy, fm, fd);

    let req = DerivativeContractsRequest {
        symbols: Some(vec!["CL".to_string()]),
        sec_type: Some("FOP".to_string()),
        expiry: Some(expiry),
        ..Default::default()
    };
    match tc.get_derivative_contracts(req).await {
        Ok(contracts) if !contracts.is_empty() => Some(contracts[0].clone()),
        _ => None,
    }
}

/// A listed US option chain slice: the chosen expiry and, for one side, the
/// strikes the exchange actually lists.
pub struct UsOptionChain {
    /// `yyyymmdd` — the shape order requests carry.
    pub expiry: String,
    /// Listed strikes for the requested side, ascending by value, wire text.
    pub strikes: Vec<String>,
}

/// Resolve a live AAPL option chain: the earliest expiry more than 14 days out,
/// then that expiry's chain filtered to `right` ("CALL" / "PUT").
///
/// Orders have to reference contracts the exchange lists. A synthesised expiry
/// ("today + 30 days") usually falls on a date Apple does not list — weeklies
/// stop roughly six weeks out — and the server answers 合约不正确 before it
/// ever looks at the price, which says nothing about the SDK.
pub async fn resolve_us_option_chain(right: &str) -> Option<UsOptionChain> {
    use tigeropen::model::quote_requests::{OptionChainItem, OptionChainRequest};

    let qc = tigeropen::quote::QuoteClient::from_config(integ_config());

    let expirations = qc.get_option_expiration(&["AAPL"], None).await.ok()?;
    let dates = expirations.first()?.dates.clone();
    let today = now_days();
    // The nearest expiry can already be expired or expiring same-day
    // (untradable), so it is skipped; if every date falls inside that window
    // the latest one is used rather than returning nothing.
    let expiry = dates
        .iter()
        .find(|d| parse_date_days(d).is_some_and(|days| days - today > 14))
        .or_else(|| dates.last())
        .cloned()?;

    let item = OptionChainItem::from_date("AAPL", &expiry).ok()?;
    let chains = qc
        .get_option_chain(OptionChainRequest::new(vec![item]))
        .await
        .ok()?;
    let chain = chains.into_iter().next()?;

    let mut strikes: Vec<(f64, String)> = Vec::new();
    for row in &chain.items {
        let leg = if right.eq_ignore_ascii_case("PUT") {
            row.put.as_ref()
        } else {
            row.call.as_ref()
        };
        let Some(leg) = leg else { continue };
        if leg.identifier.is_empty() {
            continue;
        }
        if let Ok(value) = leg.strike.parse::<f64>() {
            strikes.push((value, leg.strike.clone()));
        }
    }
    strikes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    strikes.dedup_by(|a, b| (a.0 - b.0).abs() < f64::EPSILON);
    if strikes.is_empty() {
        return None;
    }

    Some(UsOptionChain {
        expiry: expiry.replace('-', ""),
        strikes: strikes.into_iter().map(|(_, text)| text).collect(),
    })
}

/// Returns true when `market` is genuinely in its live trading session,
/// mirroring the C++ / Java / Go / C# reference helpers: fetch real market
/// status and compare against `"TRADING"`. Fails closed (`false`) on any
/// gateway error, so a lookup failure is treated as "not trading" rather
/// than risking a false-positive re-check.
pub async fn is_market_trading(qc: &tigeropen::quote::QuoteClient, market: &str) -> bool {
    match qc.get_market_state(market).await {
        Ok(states) => states.first().is_some_and(|s| {
            let status = if !s.market_status.is_empty() {
                &s.market_status
            } else {
                &s.status
            };
            status == "TRADING"
        }),
        Err(_) => false,
    }
}

/// A BUY limit price far below the current HK quote, but within the
/// exchange's price-deviation tolerance — unlike a hardcoded absolute
/// constant, this never falls too far outside whatever range the exchange
/// currently allows. Aligned to the symbol's `min_tick`, since HK's tick
/// size varies by price band and a price that isn't a clean multiple of it
/// is rejected outright.
///
/// Returns `None` when no live quote/trade-meta is available, rather than
/// guessing a price that may or may not be accepted — caller should skip.
pub async fn safe_hk_buy_price(symbol: &str) -> Option<f64> {
    use tigeropen::model::quote_requests::{BriefRequest, TradeMetasRequest};

    let qc = tigeropen::quote::QuoteClient::from_config(integ_config());

    let briefs = qc
        .get_real_time_quote(BriefRequest {
            symbols: Some(vec![symbol.to_string()]),
            ..Default::default()
        })
        .await
        .ok()?;
    let latest_price = briefs.first()?.latest_price;
    if !(latest_price > 0.0) {
        return None;
    }

    let metas = qc
        .get_trade_metas(TradeMetasRequest {
            symbols: Some(vec![symbol.to_string()]),
            ..Default::default()
        })
        .await
        .ok()?;
    let min_tick = metas.first()?.min_tick;
    if !(min_tick > 0.0) {
        return None;
    }

    let raw_price = latest_price * 0.5;
    let ticks = (raw_price / min_tick).floor();
    Some((ticks * min_tick * 10_000.0).round() / 10_000.0)
}

/// Days since the Unix epoch.
fn now_days() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64 / 86_400_000)
        .unwrap_or(0)
}

/// Parse the exchange's `yyyy-MM-dd` expiry into days since the Unix epoch.
fn parse_date_days(date: &str) -> Option<i64> {
    let mut parts = date.split('-');
    let y: i32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let d: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(days_from_civil(y, m, d))
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

/// Inverse of `civil_from_days` — converts (year, month, day) into
/// days-since-1970-01-01. Based on Howard Hinnant's date algorithms.
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y as i64 - 1 } else { y as i64 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe as i64 - 719_468
}
