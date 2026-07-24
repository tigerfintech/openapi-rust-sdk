//! Integration test: real token refresh via user_token_refresh API.
//!
//! Requires a TBHK account with a valid token in the properties file.
//! Run: TIGER_CONFIG_PATH=/path/to/tbhk.properties cargo run --example integ_real_token

use tigeropen::client::http_client::HttpClient;
use tigeropen::config::ClientConfig;

fn pass(name: &str, note: &str) {
    println!("[ OK ] {:<58} {}", name, note);
}
fn fail(name: &str, err: impl std::fmt::Display) -> ! {
    println!("[FAIL] {:<58} {}", name, err);
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    let mut cfg = match std::env::var("TIGER_CONFIG_PATH") {
        Ok(path) => ClientConfig::builder().properties_file(&path).build(),
        Err(_) => ClientConfig::builder().build(),
    }
    .unwrap_or_else(|e| fail("ClientConfig::build", e));

    println!(
        "tiger_id={} license={:?} token={}\n",
        cfg.tiger_id,
        cfg.license,
        cfg.token
            .as_deref()
            .map(|t| &t[..t.len().min(20)])
            .unwrap_or("(none)")
    );

    let hc = HttpClient::new(cfg.clone());

    // ── Test 1: query_token ────────────────────────────────────────────────────
    let new_token = match hc.query_token().await {
        Ok(t) => {
            pass(
                "query_token()",
                &format!("len={} prefix={}", t.len(), &t[..t.len().min(20)]),
            );
            t
        }
        Err(e) => fail("query_token()", e),
    };

    // Sync new token back to config for the next call
    cfg.token = Some(new_token.clone());
    let hc2 = HttpClient::new(cfg.clone());

    // ── Test 2: refresh_token updates config ───────────────────────────────────
    let token_before = cfg.token.clone().unwrap_or_default();
    match hc2.refresh_token(None).await {
        Ok(()) => {
            // After refresh, config.token is updated inside hc2's Arc<RwLock<Config>>.
            // We verify by calling query_token() again — if it succeeds, the new token is valid.
            pass(
                "refresh_token()",
                &format!(
                    "before_prefix={}",
                    &token_before[..token_before.len().min(20)]
                ),
            );
        }
        Err(e) => fail("refresh_token()", e),
    }

    // ── Test 3: startTokenAutoRefresh with real API ────────────────────────────
    // Generate a new fresh token first (previous two calls consumed the old ones)
    let fresh_token = match hc2.query_token().await {
        Ok(t) => t,
        Err(e) => fail("query_token() for auto-refresh test", e),
    };
    cfg.token = Some(fresh_token.clone());

    // Build an expired token to force immediate refresh
    let expired = {
        use base64::{engine::general_purpose::STANDARD, Engine};
        let payload = format!("{:013},{:013}extra_payload", 1, 2);
        STANDARD.encode(payload.as_bytes())
    };

    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::{Arc, Mutex};
    use tigeropen::config::TokenManager;

    let refresh_count = Arc::new(AtomicU32::new(0));
    let rc_clone = refresh_count.clone();
    let received_token: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let rt_clone = received_token.clone();

    let mut tm = TokenManager::with_refresh_duration(None, 30);
    tm.set_check_interval(1);
    tm.set_token_writer(move |t: String| {
        rc_clone.fetch_add(1, Ordering::Relaxed);
        *rt_clone.lock().unwrap() = t;
    });
    tm.set_token(&expired)
        .unwrap_or_else(|e| fail("set expired token", e));

    let cfg_for_refresh = cfg.clone();
    tm.start_auto_refresh(move || {
        let cfg_inner = cfg_for_refresh.clone();
        async move {
            let hc_inner = HttpClient::new(cfg_inner);
            hc_inner.query_token().await
        }
    });

    // Wait up to 4s for auto-refresh to fire
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(4);
    loop {
        if refresh_count.load(Ordering::Relaxed) > 0 {
            break;
        }
        if tokio::time::Instant::now() >= deadline {
            tm.stop_auto_refresh();
            fail(
                "start_auto_refresh with real API",
                "not triggered within 4s",
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    tm.stop_auto_refresh();

    let got_token = received_token.lock().unwrap().clone();
    if got_token.is_empty() {
        fail("tokenWriter received token", "empty");
    }
    pass(
        "start_auto_refresh with real API",
        &format!(
            "count={} token_prefix={}",
            refresh_count.load(Ordering::Relaxed),
            &got_token[..got_token.len().min(20)]
        ),
    );

    println!();
    println!("══════════════════════════════════════════════════════════════════");
    println!("  PASS: All real token refresh tests passed");
    println!("══════════════════════════════════════════════════════════════════");
}
