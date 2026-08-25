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
