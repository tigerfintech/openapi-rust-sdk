//! Integration tests for varint32 and proto_message modules

use tigeropen::push::varint::{decode_varint32, encode_varint32};
use tigeropen::push::proto_message::*;
use tigeropen::push::pb::socket_common::{Command, DataType};
use tigeropen::push::SubjectType;

// ===== varint32 tests =====

#[test]
fn test_varint_encode_decode_empty() {
    let data = b"";
    let encoded = encode_varint32(data);
    assert_eq!(encoded, vec![0]);
    let (msg, remaining) = decode_varint32(&encoded).unwrap();
    assert_eq!(msg, data);
    assert!(remaining.is_empty());
}

#[test]
fn test_varint_encode_decode_small() {
    let data = b"hello";
    let encoded = encode_varint32(data);
    assert_eq!(encoded[0], 5);
    assert_eq!(&encoded[1..], b"hello");
    let (msg, remaining) = decode_varint32(&encoded).unwrap();
    assert_eq!(msg, b"hello");
    assert!(remaining.is_empty());
}

#[test]
fn test_varint_encode_decode_128_bytes() {
    let data = vec![0xAB; 128];
    let encoded = encode_varint32(&data);
    assert_eq!(encoded[0], 0x80);
    assert_eq!(encoded[1], 0x01);
    let (msg, remaining) = decode_varint32(&encoded).unwrap();
    assert_eq!(msg, data.as_slice());
    assert!(remaining.is_empty());
}

#[test]
fn test_varint_decode_insufficient_header() {
    assert!(decode_varint32(&[0x80]).is_none());
}

#[test]
fn test_varint_decode_insufficient_body() {
    let mut buffer = vec![10];
    buffer.extend_from_slice(&[1, 2, 3]);
    assert!(decode_varint32(&buffer).is_none());
}

#[test]
fn test_varint_decode_with_remaining() {
    let data1 = b"abc";
    let data2 = b"xyz";
    let mut buffer = encode_varint32(data1);
    buffer.extend_from_slice(&encode_varint32(data2));

    let (msg1, remaining) = decode_varint32(&buffer).unwrap();
    assert_eq!(msg1, b"abc");
    let (msg2, remaining) = decode_varint32(remaining).unwrap();
    assert_eq!(msg2, b"xyz");
    assert!(remaining.is_empty());
}

#[test]
fn test_varint_decode_empty_buffer() {
    assert!(decode_varint32(&[]).is_none());
}

#[test]
fn test_varint_roundtrip_large() {
    let data = vec![42u8; 16384]; // 16KB
    let encoded = encode_varint32(&data);
    let (msg, remaining) = decode_varint32(&encoded).unwrap();
    assert_eq!(msg, data.as_slice());
    assert!(remaining.is_empty());
}

// ===== proto_message tests =====

#[test]
fn test_build_connect_message() {
    let msg = build_connect_message(
        "test_tiger_id", "test_sign", "rust-sdk-1.0", "1.0", 10000, 30000, false,
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
    let msg = build_subscribe_message(DataType::Quote as i32, Some("AAPL,GOOG"), None, None);
    assert_eq!(msg.command, Command::Subscribe as i32);
    assert!(msg.id > 0);
    let sub = msg.subscribe.unwrap();
    assert_eq!(sub.data_type, DataType::Quote as i32);
    assert_eq!(sub.symbols, Some("AAPL,GOOG".to_string()));
    assert!(sub.account.is_none());
    assert!(sub.market.is_none());
}

#[test]
fn test_build_subscribe_with_account() {
    let msg = build_subscribe_message(DataType::Asset as i32, None, Some("test_account"), None);
    assert_eq!(msg.command, Command::Subscribe as i32);
    let sub = msg.subscribe.unwrap();
    assert_eq!(sub.data_type, DataType::Asset as i32);
    assert!(sub.symbols.is_none());
    assert_eq!(sub.account, Some("test_account".to_string()));
}

#[test]
fn test_build_subscribe_with_market() {
    let msg = build_subscribe_message(
        DataType::StockTop as i32, Some("changeRate"), None, Some("US"),
    );
    assert_eq!(msg.command, Command::Subscribe as i32);
    let sub = msg.subscribe.unwrap();
    assert_eq!(sub.data_type, DataType::StockTop as i32);
    assert_eq!(sub.symbols, Some("changeRate".to_string()));
    assert_eq!(sub.market, Some("US".to_string()));
}

#[test]
fn test_build_unsubscribe_message() {
    let msg = build_unsubscribe_message(DataType::Asset as i32, None, Some("test_account"), None);
    assert_eq!(msg.command, Command::Unsubscribe as i32);
    let sub = msg.subscribe.unwrap();
    assert_eq!(sub.data_type, DataType::Asset as i32);
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
