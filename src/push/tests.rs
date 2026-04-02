//! PushClient 测试

use super::*;
use std::sync::{Arc, atomic::{AtomicI32, Ordering}};

fn test_config() -> crate::config::ClientConfig {
    crate::config::ClientConfig {
        tiger_id: "test_tiger_id".into(),
        private_key: "test_key".into(),
        account: "test_account".into(),
        license: None,
        language: crate::model::enums::Language::ZhCn,
        timezone: None,
        timeout: std::time::Duration::from_secs(15),
        sandbox_debug: false,
        token: None,
        token_refresh_duration: None,
        server_url: "https://openapi.tigerfintech.com/gateway".into(),
    }
}

#[test]
fn test_quote_data_roundtrip() {
    let data = QuotePushData {
        symbol: "AAPL".into(),
        latest_price: Some(150.25),
        pre_close: None, open: None, high: None, low: None,
        volume: Some(1000000), amount: None, timestamp: Some(1700000000),
    };
    let json = serde_json::to_string(&data).unwrap();
    let restored: QuotePushData = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.symbol, "AAPL");
    assert_eq!(restored.latest_price, Some(150.25));
    assert_eq!(restored.volume, Some(1000000));
}

#[test]
fn test_tick_data_roundtrip() {
    let data = TickPushData {
        symbol: "TSLA".into(), price: 250.5, volume: 500,
        tick_type: Some("BUY".into()), timestamp: Some(1700000001),
    };
    let json = serde_json::to_string(&data).unwrap();
    let restored: TickPushData = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.symbol, "TSLA");
    assert_eq!(restored.price, 250.5);
}

#[test]
fn test_depth_data_roundtrip() {
    let data = DepthPushData {
        symbol: "AAPL".into(),
        asks: Some(vec![PriceLevel { price: 150.1, volume: 100, count: Some(5) }]),
        bids: Some(vec![PriceLevel { price: 150.0, volume: 150, count: None }]),
    };
    let json = serde_json::to_string(&data).unwrap();
    let restored: DepthPushData = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.asks.unwrap()[0].price, 150.1);
}

#[test]
fn test_order_data_roundtrip() {
    let data = OrderPushData {
        account: "acc".into(), id: Some(1001), order_id: Some(2001),
        symbol: "AAPL".into(), action: Some("BUY".into()),
        order_type: Some("LMT".into()), quantity: Some(100),
        limit_price: Some(150.0), status: Some("Filled".into()),
        filled: Some(100), avg_fill_price: Some(149.95),
    };
    let json = serde_json::to_string(&data).unwrap();
    let restored: OrderPushData = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.status, Some("Filled".into()));
}

#[test]
fn test_asset_data_roundtrip() {
    let data = AssetPushData {
        account: "acc".into(), net_liquidation: Some(100000.5),
        equity_with_loan: None, cash_balance: Some(50000.25),
        buying_power: None, currency: Some("USD".into()),
    };
    let json = serde_json::to_string(&data).unwrap();
    let restored: AssetPushData = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.net_liquidation, Some(100000.5));
}

#[test]
fn test_push_message_roundtrip() {
    let quote = QuotePushData {
        symbol: "AAPL".into(), latest_price: Some(150.0),
        pre_close: None, open: None, high: None, low: None,
        volume: None, amount: None, timestamp: None,
    };
    let msg = PushMessage {
        msg_type: MessageType::Quote,
        subject: Some(SubjectType::Quote),
        data: Some(serde_json::to_value(&quote).unwrap()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let restored: PushMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.msg_type, MessageType::Quote);
    assert_eq!(restored.subject, Some(SubjectType::Quote));
}

// ===== 订阅状态管理测试 =====

#[test]
fn test_subscription_state_management() {
    let client = PushClient::new(test_config(), Some(PushClientOptions {
        auto_reconnect: Some(false), ..Default::default()
    }));
    assert_eq!(client.state(), ConnectionState::Disconnected);

    // 添加订阅
    client.add_subscription(SubjectType::Quote, &["AAPL".into(), "TSLA".into()]);
    client.add_subscription(SubjectType::Tick, &["GOOG".into()]);
    let subs = client.get_subscriptions();
    assert_eq!(subs.len(), 2);
    assert_eq!(subs[&SubjectType::Quote].len(), 2);

    // 追加订阅
    client.add_subscription(SubjectType::Quote, &["GOOG".into()]);
    let subs = client.get_subscriptions();
    assert_eq!(subs[&SubjectType::Quote].len(), 3);

    // 部分退订
    client.remove_subscription(SubjectType::Quote, Some(&["TSLA".into()]));
    let subs = client.get_subscriptions();
    assert_eq!(subs[&SubjectType::Quote].len(), 2);

    // 全部退订
    client.remove_subscription(SubjectType::Quote, None);
    let subs = client.get_subscriptions();
    assert!(!subs.contains_key(&SubjectType::Quote));
}

#[test]
fn test_account_subscription_management() {
    let client = PushClient::new(test_config(), None);

    client.add_account_sub(SubjectType::Asset);
    client.add_account_sub(SubjectType::Position);
    client.add_account_sub(SubjectType::Order);
    client.add_account_sub(SubjectType::Transaction);
    assert_eq!(client.get_account_subscriptions().len(), 4);

    client.remove_account_sub(&SubjectType::Asset);
    client.remove_account_sub(&SubjectType::Position);
    assert_eq!(client.get_account_subscriptions().len(), 2);
}

// ===== 回调测试 =====

#[test]
fn test_quote_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_quote: Some(Arc::new(move |_data| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });
    let msg = r#"{"type":"quote","subject":"quote","data":{"symbol":"AAPL","latestPrice":155.0}}"#;
    client.handle_message(msg);
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_order_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_order: Some(Arc::new(move |_data| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });
    let msg = r#"{"type":"order","data":{"account":"acc","symbol":"AAPL","status":"Filled"}}"#;
    client.handle_message(msg);
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_kickout_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_kickout: Some(Arc::new(move |_msg| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });
    let msg = r#"{"type":"kickout","data":"另一设备登录"}"#;
    client.handle_message(msg);
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_disconnect_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_disconnect: Some(Arc::new(move || {
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });
    client.disconnect();
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

// Feature: multi-language-sdks, Property 12: Protobuf 序列化 round-trip
// （简化为 JSON 序列化 round-trip）
// **Validates: Requirements 6.12**
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    fn symbol_strategy() -> impl Strategy<Value = String> {
        "[A-Z]{1,5}"
    }

    fn price_strategy() -> impl Strategy<Value = f64> {
        0.0f64..100000.0f64
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn quote_data_roundtrip(
            symbol in symbol_strategy(),
            price in price_strategy(),
            volume in 0i64..1000000000i64,
            ts in 0i64..2000000000i64,
        ) {
            let data = QuotePushData {
                symbol: symbol.clone(),
                latest_price: Some(price),
                pre_close: None, open: None, high: None, low: None,
                volume: Some(volume), amount: None, timestamp: Some(ts),
            };
            let msg = PushMessage {
                msg_type: MessageType::Quote,
                subject: Some(SubjectType::Quote),
                data: Some(serde_json::to_value(&data).unwrap()),
            };
            let json = serde_json::to_string(&msg).unwrap();
            let restored: PushMessage = serde_json::from_str(&json).unwrap();
            let d: QuotePushData = serde_json::from_value(restored.data.unwrap()).unwrap();
            prop_assert_eq!(&d.symbol, &symbol);
            prop_assert_eq!(d.volume, Some(volume));
            prop_assert_eq!(d.timestamp, Some(ts));
        }

        #[test]
        fn order_data_roundtrip(
            account in "[a-z0-9]{5,10}",
            symbol in symbol_strategy(),
            status in prop_oneof!["Submitted", "Filled", "Cancelled"],
            qty in 1i32..10000i32,
        ) {
            let data = OrderPushData {
                account: account.clone(), id: Some(1), order_id: Some(2),
                symbol: symbol.clone(), action: Some("BUY".into()),
                order_type: Some("LMT".into()), quantity: Some(qty),
                limit_price: None, status: Some(status.clone()),
                filled: None, avg_fill_price: None,
            };
            let json = serde_json::to_string(&data).unwrap();
            let restored: OrderPushData = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(&restored.account, &account);
            prop_assert_eq!(&restored.symbol, &symbol);
            prop_assert_eq!(restored.status, Some(status));
            prop_assert_eq!(restored.quantity, Some(qty));
        }
    }
}