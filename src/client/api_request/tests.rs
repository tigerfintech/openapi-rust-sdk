//! API 请求构造测试模块。

use super::*;
use proptest::prelude::*;
use std::collections::BTreeMap;

// ========== 单元测试 ==========

#[test]
fn test_new_api_request() {
    let req = ApiRequest::new("market_state", r#"{"market":"US"}"#);
    assert_eq!(req.method, "market_state");
    assert_eq!(req.biz_content, r#"{"market":"US"}"#);
}

#[test]
fn test_new_api_request_empty_biz_content() {
    let req = ApiRequest::new("market_state", "{}");
    assert_eq!(req.method, "market_state");
    assert_eq!(req.biz_content, "{}");
}

#[test]
fn test_from_params_with_map() {
    let mut params = BTreeMap::new();
    params.insert("market", "US");
    params.insert("symbol", "AAPL");
    let req = ApiRequest::from_params("quote_real_time", &params).unwrap();
    assert_eq!(req.method, "quote_real_time");
    // BTreeMap 序列化后键按字母序排列
    let parsed: serde_json::Value = serde_json::from_str(&req.biz_content).unwrap();
    assert_eq!(parsed["market"], "US");
    assert_eq!(parsed["symbol"], "AAPL");
}

#[test]
fn test_api_request_serialization() {
    let req = ApiRequest::new("market_state", r#"{"market":"US"}"#);
    let json = serde_json::to_string(&req).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["method"], "market_state");
    assert_eq!(parsed["biz_content"], r#"{"market":"US"}"#);
}

// ========== Property 11 属性测试 ==========

/// 生成有效的 API 方法名
fn valid_api_method() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("market_state".to_string()),
        Just("quote_real_time".to_string()),
        Just("kline".to_string()),
        Just("place_order".to_string()),
        Just("modify_order".to_string()),
        Just("cancel_order".to_string()),
        Just("get_position".to_string()),
        Just("get_orders".to_string()),
        Just("option_chain".to_string()),
        Just("future_exchange".to_string()),
    ]
}

/// 生成有效的业务参数（BTreeMap 保证键按字母序）
fn valid_biz_params() -> impl Strategy<Value = BTreeMap<String, String>> {
    proptest::collection::btree_map(
        "[a-z_]{1,15}",
        "[a-zA-Z0-9]{1,20}",
        0..5,
    )
}

// **Validates: Requirements 4.1-4.12, 5.1-5.12**
//
// Feature: multi-language-sdks, Property 11: API 请求构造正确性
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn api_request_construction_correctness(
        method in valid_api_method(),
        params in valid_biz_params(),
    ) {
        let req = ApiRequest::from_params(&method, &params).unwrap();
        // method 字段应与传入的方法名一致
        prop_assert_eq!(&req.method, &method);
        // biz_content 应为有效 JSON
        let parsed: BTreeMap<String, String> = serde_json::from_str(&req.biz_content).unwrap();
        // 反序列化后应与原始参数一致
        prop_assert_eq!(&parsed, &params);
    }
}
