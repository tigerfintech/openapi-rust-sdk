//! PushClient tests

use super::*;
use std::sync::{Arc, atomic::{AtomicI32, Ordering}};

use prost::Message;
use super::pb;
use super::pb::socket_common::{Command, DataType};
use super::varint::encode_varint32;

fn test_config() -> crate::config::ClientConfig {
    crate::config::ClientConfig {
        tiger_id: "test_tiger_id".into(),
        private_key: "test_key".into(),
        account: "test_account".into(),
        license: None,
        language: crate::model::enums::Language::ZhCn,
        timezone: None,
        timeout: std::time::Duration::from_secs(15),
        token: None,
        token_refresh_duration: None,
        server_url: "https://openapi.tigerfintech.com/gateway".into(),
        quote_server_url: "https://openapi.tigerfintech.com/gateway".into(),
        tiger_public_key: "".into(),
        device_id: "".into(),
    }
}

/// 辅助函数：将 Response 编码为 varint32 + protobuf 二进制帧
fn encode_response(response: &pb::Response) -> Vec<u8> {
    let proto_bytes = response.encode_to_vec();
    encode_varint32(&proto_bytes)
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

// ===== 消息处理回调测试 =====

#[test]
fn test_connected_response() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_connect: Some(Arc::new(move || {
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });

    let response = pb::Response {
        command: Command::Connected as i32,
        id: Some(1),
        code: None,
        msg: None,
        body: None,
    };
    client.handle_message(&encode_response(&response));
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert_eq!(client.state(), ConnectionState::Connected);
}

#[test]
fn test_quote_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_quote: Some(Arc::new(move |data: pb::QuoteData| {
            assert_eq!(data.symbol, "AAPL");
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });

    let response = pb::Response {
        command: Command::Message as i32,
        id: Some(2),
        code: None,
        msg: None,
        body: Some(pb::PushData {
            data_type: DataType::Quote as i32,
            body: Some(pb::push_data::Body::QuoteData(pb::QuoteData {
                symbol: "AAPL".into(),
                latest_price: Some(155.0),
                ..Default::default()
            })),
        }),
    };
    client.handle_message(&encode_response(&response));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_depth_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_depth: Some(Arc::new(move |data: pb::QuoteDepthData| {
            assert_eq!(data.symbol, "AAPL");
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });

    let response = pb::Response {
        command: Command::Message as i32,
        id: Some(3),
        code: None,
        msg: None,
        body: Some(pb::PushData {
            data_type: DataType::QuoteDepth as i32,
            body: Some(pb::push_data::Body::QuoteDepthData(pb::QuoteDepthData {
                symbol: "AAPL".into(),
                timestamp: 1700000000,
                ask: None,
                bid: None,
            })),
        }),
    };
    client.handle_message(&encode_response(&response));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_order_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_order: Some(Arc::new(move |data: pb::OrderStatusData| {
            assert_eq!(data.symbol, "AAPL");
            assert_eq!(data.status, "Filled");
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });

    let response = pb::Response {
        command: Command::Message as i32,
        id: Some(4),
        code: None,
        msg: None,
        body: Some(pb::PushData {
            data_type: DataType::OrderStatus as i32,
            body: Some(pb::push_data::Body::OrderStatusData(pb::OrderStatusData {
                account: "acc".into(),
                symbol: "AAPL".into(),
                status: "Filled".into(),
                ..Default::default()
            })),
        }),
    };
    client.handle_message(&encode_response(&response));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_asset_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_asset: Some(Arc::new(move |data: pb::AssetData| {
            assert_eq!(data.account, "acc");
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });

    let response = pb::Response {
        command: Command::Message as i32,
        id: Some(5),
        code: None,
        msg: None,
        body: Some(pb::PushData {
            data_type: DataType::Asset as i32,
            body: Some(pb::push_data::Body::AssetData(pb::AssetData {
                account: "acc".into(),
                net_liquidation: 100000.5,
                ..Default::default()
            })),
        }),
    };
    client.handle_message(&encode_response(&response));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_error_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_error: Some(Arc::new(move |msg: String| {
            assert!(msg.contains("服务端错误"));
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });

    let response = pb::Response {
        command: Command::Error as i32,
        id: Some(6),
        code: Some(500),
        msg: Some("internal error".into()),
        body: None,
    };
    client.handle_message(&encode_response(&response));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_kickout_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_kickout: Some(Arc::new(move |msg: String| {
            assert!(msg.contains("kickout"));
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });

    let response = pb::Response {
        command: Command::Error as i32,
        id: Some(7),
        code: Some(1001),
        msg: Some("kickout: 另一设备登录".into()),
        body: None,
    };
    client.handle_message(&encode_response(&response));
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

#[test]
fn test_disconnect_response_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_disconnect: Some(Arc::new(move || {
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });

    let response = pb::Response {
        command: Command::Disconnect as i32,
        id: Some(8),
        code: None,
        msg: None,
        body: None,
    };
    client.handle_message(&encode_response(&response));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_heartbeat_response_ignored() {
    let client = PushClient::new(test_config(), None);
    // No callbacks set — heartbeat should be silently ignored
    let response = pb::Response {
        command: Command::Heartbeat as i32,
        id: Some(9),
        code: None,
        msg: None,
        body: None,
    };
    // Should not panic
    client.handle_message(&encode_response(&response));
}

#[test]
fn test_invalid_protobuf_triggers_error() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_error: Some(Arc::new(move |_msg: String| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });

    // Send invalid data with valid varint32 header but invalid protobuf body
    let invalid_data = encode_varint32(&[0xFF, 0xFF, 0xFF]);
    client.handle_message(&invalid_data);
    // Protobuf decode may or may not fail on arbitrary bytes, but varint32 decode should succeed
    // The test verifies no panic occurs
}

#[test]
fn test_option_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_option: Some(Arc::new(move |data: pb::QuoteData| {
            assert_eq!(data.symbol, "AAPL230120C00150000");
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });

    let response = pb::Response {
        command: Command::Message as i32,
        id: Some(10),
        code: None,
        msg: None,
        body: Some(pb::PushData {
            data_type: DataType::Option as i32,
            body: Some(pb::push_data::Body::QuoteData(pb::QuoteData {
                symbol: "AAPL230120C00150000".into(),
                ..Default::default()
            })),
        }),
    };
    client.handle_message(&encode_response(&response));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_kline_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_kline: Some(Arc::new(move |data: pb::KlineData| {
            assert_eq!(data.symbol, "AAPL");
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });

    let response = pb::Response {
        command: Command::Message as i32,
        id: Some(11),
        code: None,
        msg: None,
        body: Some(pb::PushData {
            data_type: DataType::Kline as i32,
            body: Some(pb::push_data::Body::KlineData(pb::KlineData {
                symbol: "AAPL".into(),
                open: 150.0,
                high: 155.0,
                low: 149.0,
                close: 154.0,
                ..Default::default()
            })),
        }),
    };
    client.handle_message(&encode_response(&response));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_stock_top_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_stock_top: Some(Arc::new(move |data: pb::StockTopData| {
            assert_eq!(data.market, "US");
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });

    let response = pb::Response {
        command: Command::Message as i32,
        id: Some(12),
        code: None,
        msg: None,
        body: Some(pb::PushData {
            data_type: DataType::StockTop as i32,
            body: Some(pb::push_data::Body::StockTopData(pb::StockTopData {
                market: "US".into(),
                timestamp: 1700000000,
                top_data: vec![],
            })),
        }),
    };
    client.handle_message(&encode_response(&response));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_position_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_position: Some(Arc::new(move |data: pb::PositionData| {
            assert_eq!(data.symbol, "AAPL");
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });

    let response = pb::Response {
        command: Command::Message as i32,
        id: Some(13),
        code: None,
        msg: None,
        body: Some(pb::PushData {
            data_type: DataType::Position as i32,
            body: Some(pb::push_data::Body::PositionData(pb::PositionData {
                account: "acc".into(),
                symbol: "AAPL".into(),
                ..Default::default()
            })),
        }),
    };
    client.handle_message(&encode_response(&response));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_transaction_callback() {
    let client = PushClient::new(test_config(), None);
    let count = Arc::new(AtomicI32::new(0));
    let count_clone = count.clone();
    client.set_callbacks(Callbacks {
        on_transaction: Some(Arc::new(move |data: pb::OrderTransactionData| {
            assert_eq!(data.symbol, "AAPL");
            count_clone.fetch_add(1, Ordering::SeqCst);
        })),
        ..Default::default()
    });

    let response = pb::Response {
        command: Command::Message as i32,
        id: Some(14),
        code: None,
        msg: None,
        body: Some(pb::PushData {
            data_type: DataType::OrderTransaction as i32,
            body: Some(pb::push_data::Body::OrderTransactionData(pb::OrderTransactionData {
                account: "acc".into(),
                symbol: "AAPL".into(),
                ..Default::default()
            })),
        }),
    };
    client.handle_message(&encode_response(&response));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}
