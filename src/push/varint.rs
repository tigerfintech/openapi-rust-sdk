//! Varint32 帧编解码器
//!
//! 实现 varint32 长度前缀的编码和解码，与 Netty 的
//! `ProtobufVarint32FrameDecoder` / `ProtobufVarint32LengthFieldPrepender` 兼容。
//!
//! 编码规则：每字节低 7 位存数据，最高位（bit 7）为延续标志，最大 5 字节。

/// 在 protobuf 字节前添加 varint32 长度前缀。
///
/// 返回 `varint32(len) + data` 的完整帧。
pub fn encode_varint32(data: &[u8]) -> Vec<u8> {
    let len = data.len() as u32;
    let mut result = Vec::with_capacity(5 + data.len());

    // 编码长度为 varint32
    let mut value = len;
    loop {
        if value & !0x7F == 0 {
            result.push(value as u8);
            break;
        }
        result.push((value & 0x7F | 0x80) as u8);
        value >>= 7;
    }

    result.extend_from_slice(data);
    result
}

/// 从缓冲区解码一个 varint32 帧。
///
/// 成功时返回 `Some((message, remaining))`，其中 `message` 是完整的 protobuf 消息，
/// `remaining` 是缓冲区中剩余的数据。
///
/// 如果数据不足（varint32 未完整或消息体不够长），返回 `None`。
pub fn decode_varint32(buffer: &[u8]) -> Option<(&[u8], &[u8])> {
    let mut value: u32 = 0;
    let mut shift: u32 = 0;

    for i in 0..buffer.len().min(5) {
        let byte = buffer[i];
        value |= ((byte & 0x7F) as u32) << shift;

        if byte & 0x80 == 0 {
            // varint32 解码完成
            let header_len = i + 1;
            let msg_len = value as usize;
            let total = header_len + msg_len;

            if buffer.len() < total {
                // 消息体数据不足
                return None;
            }

            let message = &buffer[header_len..total];
            let remaining = &buffer[total..];
            return Some((message, remaining));
        }

        shift += 7;
    }

    // 数据不足以读取完整的 varint32 前缀
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_empty() {
        let data = b"";
        let encoded = encode_varint32(data);
        assert_eq!(encoded, vec![0]);

        let (msg, remaining) = decode_varint32(&encoded).unwrap();
        assert_eq!(msg, data);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_encode_decode_small() {
        let data = b"hello";
        let encoded = encode_varint32(data);
        assert_eq!(encoded[0], 5); // length prefix
        assert_eq!(&encoded[1..], b"hello");

        let (msg, remaining) = decode_varint32(&encoded).unwrap();
        assert_eq!(msg, b"hello");
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_encode_decode_128_bytes() {
        let data = vec![0xAB; 128];
        let encoded = encode_varint32(&data);
        // 128 = 0x80 → varint32: [0x80, 0x01]
        assert_eq!(encoded[0], 0x80);
        assert_eq!(encoded[1], 0x01);
        assert_eq!(&encoded[2..], data.as_slice());

        let (msg, remaining) = decode_varint32(&encoded).unwrap();
        assert_eq!(msg, data.as_slice());
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_decode_insufficient_header() {
        // 延续标志设置但没有后续字节
        let buffer = vec![0x80];
        assert!(decode_varint32(&buffer).is_none());
    }

    #[test]
    fn test_decode_insufficient_body() {
        // varint32 表示长度为 10，但只有 3 字节数据
        let mut buffer = vec![10]; // length = 10
        buffer.extend_from_slice(&[1, 2, 3]); // only 3 bytes
        assert!(decode_varint32(&buffer).is_none());
    }

    #[test]
    fn test_decode_with_remaining() {
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
    fn test_decode_empty_buffer() {
        assert!(decode_varint32(&[]).is_none());
    }

    // ===== Property-based tests using proptest =====

    use proptest::prelude::*;
    use prost::Message;

    /// Property 1: Varint32 encode/decode round-trip
    /// For any Vec<u8> data (0..10000 bytes), encode_varint32(&data) then decode_varint32
    /// should return the original data.
    /// **Validates: Requirements 2.1, 2.2**
    proptest! {
        #[test]
        fn prop_varint32_roundtrip(data in proptest::collection::vec(any::<u8>(), 0..10000)) {
            let encoded = encode_varint32(&data);
            let (decoded, remaining) = decode_varint32(&encoded).expect("decode should succeed");
            prop_assert_eq!(decoded, data.as_slice());
            prop_assert!(remaining.is_empty());
        }
    }

    /// Property 2: Varint32 chunked decode
    /// For any data and random split point, encoding then splitting should still decode
    /// correctly when all bytes are available.
    /// **Validates: Requirements 2.4**
    proptest! {
        #[test]
        fn prop_varint32_chunked_decode(
            data in proptest::collection::vec(any::<u8>(), 0..5000),
            split_pct in 0.0f64..=1.0f64,
        ) {
            let encoded = encode_varint32(&data);
            let split_point = (encoded.len() as f64 * split_pct) as usize;
            let split_point = split_point.min(encoded.len());

            let chunk1 = &encoded[..split_point];
            let chunk2 = &encoded[split_point..];

            // First chunk alone may or may not decode (depends on split point)
            // But the full buffer must always decode correctly
            let full = [chunk1, chunk2].concat();
            let (decoded, remaining) = decode_varint32(&full).expect("full buffer decode should succeed");
            prop_assert_eq!(decoded, data.as_slice());
            prop_assert!(remaining.is_empty());
        }
    }

    /// Property 3: Request message frame round-trip
    /// For any valid pb::Request, encode to protobuf → encode_varint32 → decode_varint32
    /// → pb::Request::decode should produce equivalent Request.
    /// **Validates: Requirements 11.6, 12.4**
    fn arb_request() -> impl Strategy<Value = super::super::pb::Request> {
        use super::super::pb;
        use super::super::pb::socket_common::{Command, DataType};

        // Generate one of the 5 command types with appropriate sub-messages
        prop_oneof![
            // CONNECT
            (
                ".*",
                ".*",
                ".*",
                ".*",
                any::<u32>(),
                any::<u32>(),
                any::<bool>()
            )
                .prop_map(|(tid, sign, sdk, ver, si, ri, ft)| {
                    pb::Request {
                        command: Command::Connect as i32,
                        id: 1,
                        connect: Some(pb::request::Connect {
                            tiger_id: tid,
                            sign,
                            sdk_version: sdk,
                            accept_version: Some(ver),
                            send_interval: Some(si),
                            receive_interval: Some(ri),
                            use_full_tick: Some(ft),
                        }),
                        subscribe: None,
                    }
                }),
            // HEARTBEAT
            Just(pb::Request {
                command: Command::Heartbeat as i32,
                id: 2,
                connect: None,
                subscribe: None,
            }),
            // SUBSCRIBE
            (1..=12i32, ".*", ".*", ".*", proptest::option::of(".*")).prop_map(
                |(dt, sym, acc, mkt, aqt)| {
                    pb::Request {
                        command: Command::Subscribe as i32,
                        id: 3,
                        connect: None,
                        subscribe: Some(pb::request::Subscribe {
                            data_type: dt,
                            symbols: Some(sym),
                            account: Some(acc),
                            market: Some(mkt),
                            asset_quote_type: aqt,
                        }),
                    }
                }
            ),
            // UNSUBSCRIBE
            (1..=12i32, ".*", ".*", ".*", proptest::option::of(".*")).prop_map(
                |(dt, sym, acc, mkt, aqt)| {
                    pb::Request {
                        command: Command::Unsubscribe as i32,
                        id: 4,
                        connect: None,
                        subscribe: Some(pb::request::Subscribe {
                            data_type: dt,
                            symbols: Some(sym),
                            account: Some(acc),
                            market: Some(mkt),
                            asset_quote_type: aqt,
                        }),
                    }
                }
            ),
            // DISCONNECT
            Just(pb::Request {
                command: Command::Disconnect as i32,
                id: 5,
                connect: None,
                subscribe: None,
            }),
        ]
    }

    proptest! {
        #[test]
        fn prop_request_frame_roundtrip(request in arb_request()) {
            let proto_bytes = request.encode_to_vec();
            let framed = encode_varint32(&proto_bytes);
            let (decoded_bytes, remaining) = decode_varint32(&framed)
                .expect("varint32 decode should succeed");
            prop_assert!(remaining.is_empty());
            let decoded = super::super::pb::Request::decode(decoded_bytes)
                .expect("protobuf decode should succeed");
            prop_assert_eq!(decoded, request);
        }
    }
}
