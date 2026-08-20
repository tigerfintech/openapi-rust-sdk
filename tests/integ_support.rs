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

/// Parse a `.properties` file at the path given by `TIGEROPEN_PROPS_PATH`,
/// returning `tiger_id` and `private_key` fields.
fn load_props_credentials() -> (String, String) {
    let path = std::env::var(ENV_PROPS_PATH)
        .expect("TIGEROPEN_PROPS_PATH env var required when env vars are absent");
    let props = tigeropen::config::config_parser::parse_properties_file(&path)
        .unwrap_or_else(|e| panic!("Failed to parse properties file {}: {}", path, e));
    let tiger_id = props
        .get("tiger_id")
        .unwrap_or_else(|| panic!("tiger_id not found in properties file {}", path))
        .clone();
    let private_key = props
        .get("private_key")
        .unwrap_or_else(|| panic!("private_key not found in properties file {}", path))
        .clone();
    (tiger_id, private_key)
}

/// Build a ClientConfig from env vars or a properties file.
///
/// Credential resolution order (matches Java / Python / Go SDKs):
///   1. TIGEROPEN_TIGER_ID + TIGEROPEN_PRIVATE_KEY env vars (+ optionally TIGEROPEN_ACCOUNT)
///   2. TIGEROPEN_PROPS_PATH — path to a .properties file with tiger_id/private_key
///
/// Panics with a descriptive message when credentials are missing — a
/// misconfigured env is a real error, not a skip.
pub fn integ_config() -> tigeropen::config::ClientConfig {
    use std::time::Duration;

    // Resolve tiger_id and private_key: env vars take priority, then properties file.
    let (tiger_id, private_key) =
        match (std::env::var(ENV_TIGER_ID), std::env::var(ENV_PRIVATE_KEY)) {
            (Ok(tid), Ok(pk)) => (tid, pk),
            _ => load_props_credentials(),
        };

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
