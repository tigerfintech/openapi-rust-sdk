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

pub const DEFAULT_SERVER_URL: &str = "https://openapi.tigerfintech.com/gateway";

/// Returns true when integration tests should run.
pub fn is_integ_run() -> bool {
    std::env::var(ENV_RUN_INTEG).as_deref() == Ok("true")
}

/// Returns true when credentials are available (regardless of RUN_INTEG flag).
#[allow(dead_code)]
pub fn has_creds() -> bool {
    std::env::var(ENV_TIGER_ID).is_ok() && std::env::var(ENV_PRIVATE_KEY).is_ok()
}

/// Build a ClientConfig from env vars using the builder pattern.
/// Panics with a descriptive message when credentials are missing — a
/// misconfigured env is a real error, not a skip.
pub fn integ_config() -> tigeropen::config::ClientConfig {
    use std::time::Duration;

    let tiger_id = std::env::var(ENV_TIGER_ID)
        .expect("TIGEROPEN_TIGER_ID env var required for integration tests");
    let private_key = std::env::var(ENV_PRIVATE_KEY)
        .expect("TIGEROPEN_PRIVATE_KEY env var required for integration tests");
    let account = std::env::var(ENV_ACCOUNT).unwrap_or_default();
    let server_url =
        std::env::var(ENV_SERVER_URL).unwrap_or_else(|_| DEFAULT_SERVER_URL.to_string());

    tigeropen::config::ClientConfig::builder()
        .tiger_id(tiger_id)
        .private_key(private_key)
        .account(account)
        .server_url(server_url.clone())
        .quote_server_url(server_url)
        .timeout(Duration::from_secs(30))
        .enable_dynamic_domain(false)
        .build()
        .expect("Failed to build ClientConfig for integration tests")
}
