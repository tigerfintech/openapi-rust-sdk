//! API 响应解析测试模块。

use super::*;
use proptest::prelude::*;

// ========== 单元测试 ==========

#[test]
fn test_parse_success_response() {
    let body = br#"{"code":0,"message":"success","data":{"market":"US"},"timestamp":1234567890}"#;
    let resp = parse_api_response(body).unwrap();
    assert_eq!(resp.code, 0);
    assert_eq!(resp.message, "success");
    assert!(resp.data.is_some());
    assert_eq!(resp.timestamp, Some(1234567890));
}

#[test]
fn test_parse_error_response() {
    let body = r#"{"code":1000,"message":"common_param_error","data":null,"timestamp":1234567890}"#;
    let result = parse_api_response(body.as_bytes());
    assert!(result.is_err());
    match result.unwrap_err() {
        TigerError::Api { code, message } => {
            assert_eq!(code, 1000);
            assert_eq!(message, "common_param_error");
        }
        _ => panic!("应返回 Api 错误"),
    }
}

#[test]
fn test_parse_invalid_json() {
    let body = b"not json";
    let result = parse_api_response(body);
    assert!(result.is_err());
}

#[test]
fn test_parse_response_with_null_data() {
    let body = br#"{"code":0,"message":"success","data":null,"timestamp":1234567890}"#;
    let resp = parse_api_response(body).unwrap();
    assert_eq!(resp.code, 0);
    // data 为 JSON null，serde 会解析为 Some(Value::Null)
}

#[test]
fn test_parse_response_without_data_field() {
    let body = br#"{"code":0,"message":"success"}"#;
    let resp = parse_api_response(body).unwrap();
    assert_eq!(resp.code, 0);
    assert!(resp.data.is_none());
}

// ========== Property 6 属性测试 ==========

/// 生成有效的 API 错误码（非零）
fn non_zero_error_code() -> impl Strategy<Value = i32> {
    prop_oneof![
        Just(1),
        Just(2),
        Just(4),
        Just(5),
        Just(1000),
        Just(1010),
        Just(1100),
        Just(1200),
        Just(1300),
        Just(2100),
        Just(2200),
        Just(2300),
        Just(2400),
        Just(4000),
        1..=9999i32,
    ]
    .prop_filter("非零错误码", |c| *c != 0)
}

/// 生成安全的 JSON 字符串值（不含特殊字符）
fn safe_json_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_ ]{1,30}"
}

// **Validates: Requirements 3.5, 3.6**
//
// Feature: multi-language-sdks, Property 6: API 响应解析与错误处理
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// code=0 时解析应成功返回 data
    #[test]
    fn api_response_success_returns_data(
        msg in safe_json_string(),
        data_val in safe_json_string(),
        ts in 1000000000i64..9999999999i64,
    ) {
        let json = format!(
            r#"{{"code":0,"message":"{}","data":"{}","timestamp":{}}}"#,
            msg, data_val, ts
        );
        let result = parse_api_response(json.as_bytes());
        prop_assert!(result.is_ok(), "code=0 时应解析成功");
        let resp = result.unwrap();
        prop_assert_eq!(resp.code, 0);
    }

    /// code!=0 时应返回包含对应 code 和 message 的错误
    #[test]
    fn api_response_error_returns_tiger_error(
        code in non_zero_error_code(),
        msg in safe_json_string(),
        ts in 1000000000i64..9999999999i64,
    ) {
        let json = format!(
            r#"{{"code":{},"message":"{}","data":null,"timestamp":{}}}"#,
            code, msg, ts
        );
        let result = parse_api_response(json.as_bytes());
        prop_assert!(result.is_err(), "code!=0 时应返回错误");
        match result.unwrap_err() {
            TigerError::Api { code: c, message: m } => {
                prop_assert_eq!(c, code);
                prop_assert_eq!(m, msg);
            }
            other => prop_assert!(false, "应返回 Api 错误，实际: {:?}", other),
        }
    }
}
