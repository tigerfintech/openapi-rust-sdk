//! Integration test: multiple real token refreshes via qa27 TBHK account.
//!
//! refresh_token() internally calls query_token() to get a new token and
//! updates config.token. We call it 3 times in sequence, each time starting
//! with the updated token from the previous round.
//!
//! Run: TIGER_CONFIG_PATH=/tmp/qa27_tbhk.properties cargo run --example integ_multi_refresh

use tigeropen::config::ClientConfig;
use tigeropen::client::http_client::HttpClient;

fn pass(name: &str, note: &str) {
    println!("[ OK ] {:<58} {}", name, note);
}
fn fail(name: &str, err: impl std::fmt::Display) -> ! {
    println!("[FAIL] {:<58} {}", name, err);
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    let cfg = match std::env::var("TIGER_CONFIG_PATH") {
        Ok(path) => ClientConfig::builder().properties_file(&path).build(),
        Err(_) => ClientConfig::builder().build(),
    }
    .unwrap_or_else(|e| fail("ClientConfig::build", e));

    println!(
        "tiger_id={} initial_token={}\n",
        cfg.tiger_id,
        cfg.token.as_deref().map(|t| &t[..t.len().min(20)]).unwrap_or("(none)")
    );

    let hc = HttpClient::new(cfg);

    // Each refresh_token() call queries a new token and updates config.token in-place.
    // The next call automatically uses the updated token.
    for i in 1..=3usize {
        let name = format!("refresh_token #{}", i);
        match hc.refresh_token(None).await {
            Ok(()) => pass(&name, "OK"),
            Err(e) => fail(&name, e),
        }
    }

    println!();
    println!("══════════════════════════════════════════════════════════════════");
    println!("  PASS: 3 consecutive refresh_token() calls succeeded");
    println!("══════════════════════════════════════════════════════════════════");
}
