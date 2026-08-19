//! Read-only PushClient integration tests requiring real API credentials.
//! Run with: TIGER_RUN_INTEG=true cargo test --test integ_push -- --nocapture

mod integ_support;

use std::sync::Arc;
use std::time::Duration;
use tigeropen::push::{connect, Callbacks, PushClient, PushClientOptions, SubjectType};
use tokio::sync::oneshot;

#[tokio::test]
async fn test_integ_full_tick_connection_and_subscription() {
    if !integ_support::is_integ_run() {
        return;
    }

    let config = integ_support::integ_config();
    let client = Arc::new(PushClient::new_with_full_tick(
        config,
        Some(PushClientOptions {
            auto_reconnect: Some(false),
            connect_timeout_secs: Some(15),
            ..Default::default()
        }),
        true,
    ));

    let (event_tx, event_rx) = oneshot::channel::<Result<String, String>>();
    let event_tx = Arc::new(std::sync::Mutex::new(Some(event_tx)));
    let full_tick_tx = Arc::clone(&event_tx);
    let error_tx = Arc::clone(&event_tx);
    client.set_callbacks(Callbacks {
        on_full_tick: Some(Arc::new(move |data| {
            if let Some(tx) = full_tick_tx.lock().unwrap().take() {
                let _ = tx.send(Ok(data.symbol));
            }
        })),
        on_error: Some(Arc::new(move |message| {
            if let Some(tx) = error_tx.lock().unwrap().take() {
                let _ = tx.send(Err(message));
            }
        })),
        ..Default::default()
    });

    connect(&client)
        .await
        .expect("full-tick push connection should authenticate");
    assert!(
        client.subscribe(
            &SubjectType::Tick,
            Some("AAPL,NVDA,TSLA,SPY,QQQ"),
            None,
            None,
        ),
        "full-tick subscription request should be sent"
    );

    match tokio::time::timeout(Duration::from_secs(45), event_rx).await {
        Ok(Ok(Ok(symbol))) => {
            assert!(["AAPL", "NVDA", "TSLA", "SPY", "QQQ"].contains(&symbol.as_str()))
        }
        Ok(Ok(Err(message))) => panic!("push server rejected full tick: {message}"),
        Ok(Err(_)) => panic!("full-tick callback channel closed unexpectedly"),
        Err(_) => panic!("no full-tick payload arrived within 45 seconds"),
    }

    client.disconnect();
}
