//! Protobuf 消息构建器
//!
//! 对齐 Java SDK 的 `ProtoMessageUtil.java`，提供构建各种 Protobuf Request 消息的工具函数。
//! 所有构建函数共享一个原子递增的 `REQUEST_ID` 计数器，为每个请求分配唯一 ID。

use std::sync::atomic::{AtomicU32, Ordering};

use super::pb::{
    self,
    socket_common::{Command, DataType},
};
use super::push_message::SubjectType;

static REQUEST_ID: AtomicU32 = AtomicU32::new(0);

/// 获取下一个递增的请求 ID
fn next_request_id() -> u32 {
    REQUEST_ID.fetch_add(1, Ordering::SeqCst) + 1
}

/// 构建 CONNECT 请求消息
pub fn build_connect_message(
    tiger_id: &str,
    sign: &str,
    sdk_version: &str,
    accept_version: &str,
    send_interval: u32,
    receive_interval: u32,
    use_full_tick: bool,
) -> pb::Request {
    pb::Request {
        command: Command::Connect as i32,
        id: next_request_id(),
        connect: Some(pb::request::Connect {
            tiger_id: tiger_id.to_string(),
            sign: sign.to_string(),
            sdk_version: sdk_version.to_string(),
            accept_version: Some(accept_version.to_string()),
            send_interval: Some(send_interval),
            receive_interval: Some(receive_interval),
            use_full_tick: Some(use_full_tick),
        }),
        subscribe: None,
    }
}

/// 构建 HEARTBEAT 请求消息
pub fn build_heartbeat_message() -> pb::Request {
    pb::Request {
        command: Command::Heartbeat as i32,
        id: next_request_id(),
        connect: None,
        subscribe: None,
    }
}

/// 构建 SUBSCRIBE 请求消息
pub fn build_subscribe_message(
    data_type: i32,
    symbols: Option<&str>,
    account: Option<&str>,
    market: Option<&str>,
) -> pb::Request {
    pb::Request {
        command: Command::Subscribe as i32,
        id: next_request_id(),
        connect: None,
        subscribe: Some(pb::request::Subscribe {
            data_type,
            symbols: symbols.map(|s| s.to_string()),
            account: account.map(|s| s.to_string()),
            market: market.map(|s| s.to_string()),
        }),
    }
}

/// 构建 UNSUBSCRIBE 请求消息
pub fn build_unsubscribe_message(
    data_type: i32,
    symbols: Option<&str>,
    account: Option<&str>,
    market: Option<&str>,
) -> pb::Request {
    pb::Request {
        command: Command::Unsubscribe as i32,
        id: next_request_id(),
        connect: None,
        subscribe: Some(pb::request::Subscribe {
            data_type,
            symbols: symbols.map(|s| s.to_string()),
            account: account.map(|s| s.to_string()),
            market: market.map(|s| s.to_string()),
        }),
    }
}

/// 构建 DISCONNECT 请求消息
pub fn build_disconnect_message() -> pb::Request {
    pb::Request {
        command: Command::Disconnect as i32,
        id: next_request_id(),
        connect: None,
        subscribe: None,
    }
}

/// 将 SubjectType 映射到 SocketCommon.DataType 整数值
pub fn subject_to_data_type(subject: &SubjectType) -> i32 {
    match subject {
        SubjectType::Quote => DataType::Quote as i32,
        SubjectType::Option => DataType::Option as i32,
        SubjectType::Future => DataType::Future as i32,
        SubjectType::Depth => DataType::QuoteDepth as i32,
        SubjectType::Tick | SubjectType::FullTick => DataType::TradeTick as i32,
        SubjectType::Asset => DataType::Asset as i32,
        SubjectType::Position => DataType::Position as i32,
        SubjectType::Order => DataType::OrderStatus as i32,
        SubjectType::Transaction => DataType::OrderTransaction as i32,
        SubjectType::StockTop => DataType::StockTop as i32,
        SubjectType::OptionTop => DataType::OptionTop as i32,
        SubjectType::Kline => DataType::Kline as i32,
        SubjectType::QuoteBbo => DataType::Quote as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_connect_message() {
        let msg = build_connect_message(
            "test_tiger_id",
            "test_sign",
            "rust-sdk-1.0",
            "1.0",
            10000,
            30000,
            false,
        );
        assert_eq!(msg.command, Command::Connect as i32);
        assert!(msg.id > 0);

        let connect = msg.connect.unwrap();
        assert_eq!(connect.tiger_id, "test_tiger_id");
        assert_eq!(connect.sign, "test_sign");
        assert_eq!(connect.sdk_version, "rust-sdk-1.0");
        assert_eq!(connect.accept_version, Some("1.0".to_string()));
        assert_eq!(connect.send_interval, Some(10000));
        assert_eq!(connect.receive_interval, Some(30000));
        assert_eq!(connect.use_full_tick, Some(false));
        assert!(msg.subscribe.is_none());
    }

    #[test]
    fn test_build_heartbeat_message() {
        let msg = build_heartbeat_message();
        assert_eq!(msg.command, Command::Heartbeat as i32);
        assert!(msg.id > 0);
        assert!(msg.connect.is_none());
        assert!(msg.subscribe.is_none());
    }

    #[test]
    fn test_build_subscribe_message() {
        let msg = build_subscribe_message(
            DataType::Quote as i32,
            Some("AAPL,GOOG"),
            None,
            None,
        );
        assert_eq!(msg.command, Command::Subscribe as i32);
        assert!(msg.id > 0);
        assert!(msg.connect.is_none());

        let sub = msg.subscribe.unwrap();
        assert_eq!(sub.data_type, DataType::Quote as i32);
        assert_eq!(sub.symbols, Some("AAPL,GOOG".to_string()));
        assert!(sub.account.is_none());
        assert!(sub.market.is_none());
    }

    #[test]
    fn test_build_unsubscribe_message() {
        let msg = build_unsubscribe_message(
            DataType::Asset as i32,
            None,
            Some("test_account"),
            None,
        );
        assert_eq!(msg.command, Command::Unsubscribe as i32);
        assert!(msg.id > 0);

        let sub = msg.subscribe.unwrap();
        assert_eq!(sub.data_type, DataType::Asset as i32);
        assert!(sub.symbols.is_none());
        assert_eq!(sub.account, Some("test_account".to_string()));
    }

    #[test]
    fn test_build_disconnect_message() {
        let msg = build_disconnect_message();
        assert_eq!(msg.command, Command::Disconnect as i32);
        assert!(msg.id > 0);
        assert!(msg.connect.is_none());
        assert!(msg.subscribe.is_none());
    }

    #[test]
    fn test_request_id_increments() {
        let msg1 = build_heartbeat_message();
        let msg2 = build_heartbeat_message();
        let msg3 = build_heartbeat_message();
        assert!(msg2.id > msg1.id);
        assert!(msg3.id > msg2.id);
    }

    #[test]
    fn test_subject_to_data_type_mapping() {
        assert_eq!(subject_to_data_type(&SubjectType::Quote), DataType::Quote as i32);
        assert_eq!(subject_to_data_type(&SubjectType::Option), DataType::Option as i32);
        assert_eq!(subject_to_data_type(&SubjectType::Future), DataType::Future as i32);
        assert_eq!(subject_to_data_type(&SubjectType::Depth), DataType::QuoteDepth as i32);
        assert_eq!(subject_to_data_type(&SubjectType::Tick), DataType::TradeTick as i32);
        assert_eq!(subject_to_data_type(&SubjectType::FullTick), DataType::TradeTick as i32);
        assert_eq!(subject_to_data_type(&SubjectType::Asset), DataType::Asset as i32);
        assert_eq!(subject_to_data_type(&SubjectType::Position), DataType::Position as i32);
        assert_eq!(subject_to_data_type(&SubjectType::Order), DataType::OrderStatus as i32);
        assert_eq!(subject_to_data_type(&SubjectType::Transaction), DataType::OrderTransaction as i32);
        assert_eq!(subject_to_data_type(&SubjectType::StockTop), DataType::StockTop as i32);
        assert_eq!(subject_to_data_type(&SubjectType::OptionTop), DataType::OptionTop as i32);
        assert_eq!(subject_to_data_type(&SubjectType::Kline), DataType::Kline as i32);
    }

    // ===== Property-based tests using proptest =====

    use proptest::prelude::*;

    /// Property 4: Request ID strictly incrementing
    /// Build N consecutive messages, each ID > previous ID.
    /// **Validates: Requirements 3.4**
    proptest! {
        #[test]
        fn prop_request_id_strictly_incrementing(n in 2..50usize) {
            let mut ids = Vec::with_capacity(n);
            for _ in 0..n {
                let msg = build_heartbeat_message();
                ids.push(msg.id);
            }
            for i in 1..ids.len() {
                prop_assert!(ids[i] > ids[i - 1],
                    "ID {} (={}) should be > ID {} (={})", i, ids[i], i - 1, ids[i - 1]);
            }
        }
    }

    /// Valid DataType values for subscribe/unsubscribe (1..=12)
    fn arb_data_type() -> impl Strategy<Value = i32> {
        (1..=12i32)
    }

    /// Property 5: Subscribe/Unsubscribe message completeness
    /// For any valid data_type i32, symbols, account, market — build_subscribe_message
    /// and build_unsubscribe_message fields should match input.
    /// **Validates: Requirements 5.1, 5.2, 5.3, 5.4, 11.3, 11.4**
    proptest! {
        #[test]
        fn prop_subscribe_message_completeness(
            data_type in arb_data_type(),
            symbols in proptest::option::of("\\PC{1,100}"),
            account in proptest::option::of("\\PC{1,50}"),
            market in proptest::option::of("\\PC{1,20}"),
        ) {
            // Test subscribe
            let sub_msg = build_subscribe_message(
                data_type,
                symbols.as_deref(),
                account.as_deref(),
                market.as_deref(),
            );
            prop_assert_eq!(sub_msg.command, Command::Subscribe as i32);
            prop_assert!(sub_msg.id > 0);
            prop_assert!(sub_msg.connect.is_none());
            let sub = sub_msg.subscribe.as_ref().unwrap();
            prop_assert_eq!(sub.data_type, data_type);
            prop_assert_eq!(&sub.symbols, &symbols.as_ref().map(|s| s.to_string()));
            prop_assert_eq!(&sub.account, &account.as_ref().map(|s| s.to_string()));
            prop_assert_eq!(&sub.market, &market.as_ref().map(|s| s.to_string()));

            // Test unsubscribe
            let unsub_msg = build_unsubscribe_message(
                data_type,
                symbols.as_deref(),
                account.as_deref(),
                market.as_deref(),
            );
            prop_assert_eq!(unsub_msg.command, Command::Unsubscribe as i32);
            prop_assert!(unsub_msg.id > 0);
            prop_assert!(unsub_msg.connect.is_none());
            let unsub = unsub_msg.subscribe.as_ref().unwrap();
            prop_assert_eq!(unsub.data_type, data_type);
            prop_assert_eq!(&unsub.symbols, &symbols.as_ref().map(|s| s.to_string()));
            prop_assert_eq!(&unsub.account, &account.as_ref().map(|s| s.to_string()));
            prop_assert_eq!(&unsub.market, &market.as_ref().map(|s| s.to_string()));
        }
    }

    /// Property 6: Connect message completeness
    /// For any valid tiger_id, sign, etc. — build_connect_message fields should match input.
    /// **Validates: Requirements 3.2, 11.1**
    proptest! {
        #[test]
        fn prop_connect_message_completeness(
            tiger_id in "\\PC{1,50}",
            sign in "\\PC{1,100}",
            sdk_version in "\\PC{1,30}",
            accept_version in "\\PC{1,10}",
            send_interval in any::<u32>(),
            receive_interval in any::<u32>(),
            use_full_tick in any::<bool>(),
        ) {
            let msg = build_connect_message(
                &tiger_id,
                &sign,
                &sdk_version,
                &accept_version,
                send_interval,
                receive_interval,
                use_full_tick,
            );
            prop_assert_eq!(msg.command, Command::Connect as i32);
            prop_assert!(msg.id > 0);
            prop_assert!(msg.subscribe.is_none());

            let connect = msg.connect.as_ref().unwrap();
            prop_assert_eq!(&connect.tiger_id, &tiger_id);
            prop_assert_eq!(&connect.sign, &sign);
            prop_assert_eq!(&connect.sdk_version, &sdk_version);
            prop_assert_eq!(connect.accept_version.as_deref(), Some(accept_version.as_str()));
            prop_assert_eq!(connect.send_interval, Some(send_interval));
            prop_assert_eq!(connect.receive_interval, Some(receive_interval));
            prop_assert_eq!(connect.use_full_tick, Some(use_full_tick));
        }
    }
}
