//! Integration test: real API connectivity + token refresh mechanism
//!
//! Run: cargo run --example integ_token_refresh
//! Override server: TIGER_SERVER_URL=<url> cargo run --example integ_token_refresh

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tigeropen::config::{ClientConfig, TokenManager};

fn pass(name: &str, note: &str) {
    println!("[ OK ] {:<58} {}", name, note);
}
fn fail(name: &str, err: impl std::fmt::Display) -> ! {
    println!("[FAIL] {:<58} {}", name, err);
    std::process::exit(1);
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn make_expired_token() -> String {
    // Token format (matches TokenManager::should_token_refresh):
    // base64( "{gen_ts_ms:13},{expire_ts_ms:13}{rest...}" )
    // The first 27 bytes after decode contain "gen_ts_ms,expire_ts_ms" (both 13-digit ms timestamps).
    // Using gen_ts=1 (epoch+1ms) makes the token always expired regardless of refresh_duration.
    let payload = format!("{:013},{:013}extra_payload_data", 1, 2);
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(payload.as_bytes())
}

#[tokio::main]
async fn main() {
    let server_url = std::env::var("TIGER_SERVER_URL").unwrap_or_default();

    let mut cfg = match std::env::var("TIGER_CONFIG_PATH") {
        Ok(path) => ClientConfig::builder().properties_file(&path).build(),
        Err(_) => ClientConfig::builder().build(),
    }
    .unwrap_or_else(|e| fail("ClientConfig::build", e));

    if !server_url.is_empty() {
        cfg.server_url = server_url;
    }
    println!("tiger_id={} server={}\n", cfg.tiger_id, cfg.server_url);

    let hc = tigeropen::client::http_client::HttpClient::new(cfg.clone());

    // ── Test 1: Connectivity ──────────────────────────────────────────────────
    match hc.execute("market_state", r#"{"market":"US"}"#).await {
        Ok(body) => {
            let code: i64 = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| v["code"].as_i64())
                .unwrap_or(-1);
            pass("market_state", &format!("code={}", code));
        }
        Err(e) => fail("market_state", e),
    }

    // ── Test 2: TokenManager auto-refresh ─────────────────────────────────────
    let want_token = format!("refreshed_token_{}", now_secs());
    let want_clone = want_token.clone();
    let refresh_count = Arc::new(AtomicU32::new(0));
    let refresh_count_clone = refresh_count.clone();
    let received_token: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let received_clone = received_token.clone();
    let expired_tok = make_expired_token();
    let expired_tok_clone = expired_tok.clone();

    let mut tm = TokenManager::new();
    tm.set_refresh_duration(1);
    tm.set_check_interval(1);
    tm.set_token_writer(move |t: String| {
        if t == expired_tok_clone {
            return;
        }
        refresh_count_clone.fetch_add(1, Ordering::Relaxed);
        *received_clone.lock().unwrap() = t;
    });
    tm.set_token(&expired_tok)
        .unwrap_or_else(|e| fail("set_token", e));
    tm.start_auto_refresh(move || {
        let tok = want_clone.clone();
        async move { Ok(tok) }
    });

    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    loop {
        if refresh_count.load(Ordering::Relaxed) > 0 {
            break;
        }
        if tokio::time::Instant::now() >= deadline {
            tm.stop_auto_refresh();
            fail("start_auto_refresh", "not triggered within 3s");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    tm.stop_auto_refresh();

    let got = received_token.lock().unwrap().clone();
    if got != want_token {
        fail(
            "token_writer value",
            &format!("got {:?}, want {:?}", got, want_token),
        );
    }
    let current = tm.get_token();
    if current != want_token {
        fail("get_token", &format!("got {:?}", current));
    }
    pass(
        "TokenManager.start_auto_refresh + token_writer",
        &format!("count={}", refresh_count.load(Ordering::Relaxed)),
    );

    // ── Test 3: Double start_auto_refresh — no goroutine leak ─────────────────
    let call_count = Arc::new(AtomicU32::new(0));
    let cc1 = call_count.clone();
    let cc2 = call_count.clone();

    let mut tm3 = TokenManager::new();
    tm3.set_refresh_duration(1);
    tm3.set_check_interval(1);
    tm3.set_token(&make_expired_token()).ok();
    tm3.start_auto_refresh(move || {
        cc1.fetch_add(1, Ordering::Relaxed);
        async { Ok("tok3a".to_string()) }
    });
    tm3.start_auto_refresh(move || {
        cc2.fetch_add(1, Ordering::Relaxed);
        async { Ok("tok3b".to_string()) }
    });

    let deadline3 = tokio::time::Instant::now() + Duration::from_secs(3);
    loop {
        if call_count.load(Ordering::Relaxed) > 0 {
            break;
        }
        if tokio::time::Instant::now() >= deadline3 {
            tm3.stop_auto_refresh();
            fail("double start_auto_refresh", "not triggered within 3s");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    tm3.stop_auto_refresh();
    let count_at_stop = call_count.load(Ordering::Relaxed);
    tokio::time::sleep(Duration::from_millis(500)).await;

    if count_at_stop == 0 {
        fail("double start_auto_refresh", "never called");
    }
    let count_after = call_count.load(Ordering::Relaxed);
    if count_after != count_at_stop {
        fail(
            "goroutine leak",
            &format!("count grew after stop: {} → {}", count_at_stop, count_after),
        );
    }
    pass(
        "double start_auto_refresh + stop (no leak)",
        &format!("calls={}", count_at_stop),
    );

    println!();
    println!("══════════════════════════════════════════════════════════════════");
    println!("  PASS: All integration tests passed");
    println!("══════════════════════════════════════════════════════════════════");
}
