//! HttpClient 测试模块。
//! 包含 Property 13/14/15 属性测试和单元测试。

use super::*;
use std::time::Duration;
use proptest::prelude::*;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::method;
use rsa::RsaPrivateKey;
use rsa::pkcs8::EncodePrivateKey;
use rsa::pkcs8::LineEnding;
use std::sync::OnceLock;

/// 缓存测试用 RSA 私钥（避免每次测试都生成，太慢）
fn cached_test_private_key() -> &'static str {
    static KEY: OnceLock<String> = OnceLock::new();
    KEY.get_or_init(|| {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("生成密钥失败");
        private_key
            .to_pkcs8_pem(LineEnding::LF)
            .expect("编码 PKCS#8 PEM 失败")
            .to_string()
    })
}

/// Create test ClientConfig
fn test_config(server_url: &str) -> ClientConfig {
    ClientConfig {
        tiger_id: "test_tiger_id".to_string(),
        private_key: cached_test_private_key().to_string(),
        account: "DU123456".to_string(),
        license: None,
        language: crate::model::enums::Language::ZhCn,
        timezone: None,
        timeout: Duration::from_secs(5),
        token: None,
        token_refresh_duration: None,
        server_url: server_url.to_string(),
        quote_server_url: server_url.to_string(),
        tiger_public_key: "".to_string(),
        device_id: "".to_string(),
    }
}

// ========== 单元测试 ==========

#[test]
fn test_user_agent() {
    assert_eq!(HttpClient::user_agent(), "openapi-rust-sdk-0.1.0");
}

#[test]
fn test_execute_rejects_empty_api_method() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let config = test_config("http://localhost:1234");
        let client = HttpClient::new(config);
        let result = client.execute("", r#"{"market":"US"}"#).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TigerError::Config(msg) => assert!(msg.contains("api_method")),
            other => panic!("应返回 Config 错误，实际: {:?}", other),
        }
    });
}

#[test]
fn test_execute_rejects_invalid_json() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let config = test_config("http://localhost:1234");
        let client = HttpClient::new(config);
        let result = client.execute("market_state", "not json").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TigerError::Config(msg) => assert!(msg.contains("JSON")),
            other => panic!("应返回 Config 错误，实际: {:?}", other),
        }
    });
}

#[tokio::test]
async fn test_execute_sends_request_and_returns_raw_response() {
    let mock_server = MockServer::start().await;
    let response_body = r#"{"code":0,"message":"success","data":{"market":"US"}}"#;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let client = HttpClient::new(config);
    let result = client.execute("market_state", r#"{"market":"US"}"#).await;
    assert!(result.is_ok());
    let body = result.unwrap();
    assert_eq!(body, response_body);
}

#[tokio::test]
async fn test_execute_request_parses_success_response() {
    let mock_server = MockServer::start().await;
    let response_body = r#"{"code":0,"message":"success","data":{"market":"US"},"timestamp":1234567890}"#;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let client = HttpClient::new(config);
    let req = ApiRequest::new("market_state", r#"{"market":"US"}"#);
    let result = client.execute_request(&req).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.code, 0);
    assert_eq!(resp.message, "success");
}

#[tokio::test]
async fn test_execute_request_returns_api_error() {
    let mock_server = MockServer::start().await;
    let response_body = r#"{"code":1010,"message":"参数错误","data":null,"timestamp":1234567890}"#;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let client = HttpClient::new(config);
    let req = ApiRequest::new("market_state", r#"{"market":"US"}"#);
    let result = client.execute_request(&req).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        TigerError::Api { code, message } => {
            assert_eq!(code, 1010);
            assert_eq!(message, "参数错误");
        }
        other => panic!("应返回 Api 错误，实际: {:?}", other),
    }
}

// ========== Property 13 属性测试 ==========

/// 生成有效的 API 方法名
fn valid_api_method() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("market_state".to_string()),
        Just("quote_real_time".to_string()),
        Just("kline".to_string()),
        Just("place_order".to_string()),
        Just("get_position".to_string()),
        Just("get_orders".to_string()),
    ]
}

/// 生成有效的 biz_content JSON 字符串
fn valid_biz_content_json() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r#"{"market":"US"}"#.to_string()),
        Just(r#"{"symbol":"AAPL"}"#.to_string()),
        Just(r#"{"market":"HK","symbol":"00700"}"#.to_string()),
        Just(r#"{}"#.to_string()),
        Just(r#"{"limit":100}"#.to_string()),
    ]
}

// **Validates: Requirements 15.3, 15.8**
//
// Feature: multi-language-sdks, Property 13: Generic execute 请求构造正确性
proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn generic_execute_request_construction(
        api_method in valid_api_method(),
        biz_content in valid_biz_content_json(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mock_server = MockServer::start().await;
            let response_body = r#"{"code":0,"message":"success","data":null}"#;

            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
                .expect(1)
                .mount(&mock_server)
                .await;

            let config = test_config(&mock_server.uri());
            let client = HttpClient::new(config);
            let result = client.execute(&api_method, &biz_content).await;
            prop_assert!(result.is_ok(), "请求应成功");

            // 验证请求已发送
            let received = mock_server.received_requests().await.unwrap();
            prop_assert_eq!(received.len(), 1, "应发送恰好一个请求");

            // 解析请求体，验证公共参数
            let req_body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
            prop_assert_eq!(req_body["method"].as_str().unwrap(), api_method.as_str(),
                "method 字段应等于传入的 api_method");
            prop_assert_eq!(req_body["biz_content"].as_str().unwrap(), biz_content.as_str(),
                "biz_content 字段应等于传入的 request_json");
            prop_assert_eq!(req_body["tiger_id"].as_str().unwrap(), "test_tiger_id",
                "tiger_id 应正确");
            prop_assert_eq!(req_body["charset"].as_str().unwrap(), "UTF-8",
                "charset 应为 UTF-8");
            prop_assert_eq!(req_body["sign_type"].as_str().unwrap(), "RSA",
                "sign_type 应为 RSA");
            prop_assert_eq!(req_body["version"].as_str().unwrap(), "2.0",
                "version 应为 2.0");
            prop_assert!(req_body["sign"].as_str().is_some(),
                "应包含 sign 字段");
            prop_assert!(req_body["timestamp"].as_str().is_some(),
                "应包含 timestamp 字段");

            Ok(())
        })?;
    }
}

// ========== Property 14 属性测试 ==========

/// 生成各种有效的 JSON 响应字符串
fn valid_response_json() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r#"{"code":0,"message":"success","data":{"market":"US"}}"#.to_string()),
        Just(r#"{"code":0,"message":"ok","data":[1,2,3]}"#.to_string()),
        Just(r#"{"code":0,"message":"","data":null}"#.to_string()),
        Just(r#"{"code":1010,"message":"error","data":null}"#.to_string()),
        Just(r#"{"code":0,"message":"success","data":{"items":[{"id":1},{"id":2}]}}"#.to_string()),
    ]
}

// **Validates: Requirements 15.4**
//
// Feature: multi-language-sdks, Property 14: Generic execute 响应原始透传
proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn generic_execute_response_passthrough(
        response_json in valid_response_json(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mock_server = MockServer::start().await;

            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(200).set_body_string(&response_json))
                .mount(&mock_server)
                .await;

            let config = test_config(&mock_server.uri());
            let client = HttpClient::new(config);
            let result = client.execute("market_state", r#"{"market":"US"}"#).await;
            prop_assert!(result.is_ok(), "请求应成功");
            let body = result.unwrap();
            // 返回的字符串应与服务器返回的原始 JSON 完全一致
            prop_assert_eq!(&body, &response_json,
                "返回的响应应与服务器原始响应完全一致");

            Ok(())
        })?;
    }
}

// ========== Property 15 属性测试 ==========

/// 生成无效的 JSON 字符串
fn invalid_json_string() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("not json".to_string()),
        Just("{invalid}".to_string()),
        Just("".to_string()),
        Just("{key: value}".to_string()),
        Just("[1,2,".to_string()),
        Just(r#"{"unclosed": "string"#.to_string()),
        Just("undefined".to_string()),
        Just("NaN".to_string()),
    ]
}

// **Validates: Requirements 15.5, 15.6**
//
// Feature: multi-language-sdks, Property 15: Generic execute 无效 JSON 拒绝
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn generic_execute_rejects_invalid_json(
        invalid_json in invalid_json_string(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let config = test_config("http://localhost:1234");
            let client = HttpClient::new(config);
            let result = client.execute("market_state", &invalid_json).await;
            // 应返回参数错误，不发送任何 HTTP 请求
            prop_assert!(result.is_err(), "无效 JSON 应返回错误");
            match result.unwrap_err() {
                TigerError::Config(msg) => {
                    prop_assert!(msg.contains("JSON"), "错误消息应提及 JSON: {}", msg);
                }
                other => {
                    prop_assert!(false, "应返回 Config 错误，实际: {:?}", other);
                }
            }

            Ok(())
        })?;
    }
}

// ========== Authorization 头测试 ==========

#[tokio::test]
async fn test_authorization_header_with_token() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"code":0,"message":"success","data":null}"#,
        ))
        .mount(&mock_server)
        .await;

    let mut config = test_config(&mock_server.uri());
    config.token = Some("my_test_token_value".to_string());
    let client = HttpClient::new(config);
    let result = client.execute("market_state", r#"{"market":"US"}"#).await;
    assert!(result.is_ok());

    let received = mock_server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);
    let auth_header: wiremock::http::HeaderName = "Authorization".into();
    let auth = received[0]
        .headers
        .get(&auth_header)
        .expect("应包含 Authorization 头");
    assert_eq!(auth.as_str(), "my_test_token_value");
}

#[tokio::test]
async fn test_no_authorization_header_without_token() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"code":0,"message":"success","data":null}"#,
        ))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    // config.token 默认为 None
    let client = HttpClient::new(config);
    let result = client.execute("market_state", r#"{"market":"US"}"#).await;
    assert!(result.is_ok());

    let received = mock_server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);
    let auth_header: wiremock::http::HeaderName = "Authorization".into();
    assert!(
        received[0].headers.get(&auth_header).is_none(),
        "未设置 Token 时不应携带 Authorization 头"
    );
}
