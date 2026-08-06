//! HttpClient 测试模块。
//! 包含 Property 13/14/15 属性测试和单元测试。

use super::*;
use proptest::prelude::*;
use rsa::pkcs8::EncodePrivateKey;
use rsa::pkcs8::LineEnding;
use rsa::RsaPrivateKey;
use std::sync::OnceLock;
use std::time::Duration;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

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
        secret_key: None,
        license: None,
        language: crate::model::enums::Language::ZhCn,
        timezone: None,
        timeout: Duration::from_secs(5),
        token: None,
        token_refresh_duration: None,
        token_check_interval: None,
        token_writer: None,
        token_loader: None,
        server_url: server_url.to_string(),
        quote_server_url: server_url.to_string(),
        tiger_public_key: "".to_string(),
        device_id: "".to_string(),
    }
}

// ========== 单元测试 ==========

#[test]
fn test_user_agent() {
    assert_eq!(
        HttpClient::user_agent(),
        format!("openapi-rust-sdk-{}", crate::VERSION)
    );
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
    let response_body =
        r#"{"code":0,"message":"success","data":{"market":"US"},"timestamp":1234567890}"#;

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
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"code":0,"message":"success","data":null}"#),
        )
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
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"code":0,"message":"success","data":null}"#),
        )
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

// ========== Token Refresh 测试 ==========

#[tokio::test]
async fn test_query_token_success() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"code":0,"message":"success","data":{"token":"new_token_abc123"}}"#,
        ))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let client = HttpClient::new(config);
    let token = client.query_token().await.unwrap();
    assert_eq!(token, "new_token_abc123");
}

#[tokio::test]
async fn test_query_token_api_error() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"code":40001,"message":"unauthorized","data":null}"#),
        )
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let client = HttpClient::new(config);
    assert!(client.query_token().await.is_err());
}

#[tokio::test]
async fn test_query_token_empty_token() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"code":0,"message":"success","data":{"token":""}}"#),
        )
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let client = HttpClient::new(config);
    assert!(client.query_token().await.is_err());
}

#[tokio::test]
async fn test_refresh_token_updates_config() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(
                r#"{"code":0,"message":"success","data":{"token":"refreshed_xyz"}}"#,
            ),
        )
        .mount(&mock_server)
        .await;

    let mut config = test_config(&mock_server.uri());
    config.token = Some("old_token".to_string());
    let client = HttpClient::new(config);
    client.refresh_token(None).await.unwrap();
    assert_eq!(
        client.config.read().unwrap().token,
        Some("refreshed_xyz".to_string())
    );
}

#[tokio::test]
async fn test_refresh_token_persists_to_file() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"code":0,"message":"success","data":{"token":"persisted_token"}}"#,
        ))
        .mount(&mock_server)
        .await;

    let tmpdir = std::env::temp_dir();
    let token_file = tmpdir.join("rust_http_client_token_test.properties");
    let tm =
        crate::config::token_manager::TokenManager::with_file_path(token_file.to_str().unwrap());

    let config = test_config(&mock_server.uri());
    let client = HttpClient::new(config);
    client.refresh_token(Some(&tm)).await.unwrap();

    let content = std::fs::read_to_string(&token_file).unwrap();
    assert_eq!(content, "token=persisted_token\n");
    std::fs::remove_file(&token_file).ok();
}

#[tokio::test]
async fn test_refresh_token_triggers_writer() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(
                r#"{"code":0,"message":"success","data":{"token":"callback_token"}}"#,
            ),
        )
        .mount(&mock_server)
        .await;

    use std::sync::{Arc, Mutex};
    let captured: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let captured_clone = Arc::clone(&captured);

    let tmpdir = std::env::temp_dir();
    let token_file = tmpdir.join("rust_http_client_writer_test.properties");
    let mut tm =
        crate::config::token_manager::TokenManager::with_file_path(token_file.to_str().unwrap());
    tm.set_token_writer(move |t| {
        *captured_clone.lock().unwrap() = t;
    });

    let config = test_config(&mock_server.uri());
    let client = HttpClient::new(config);
    client.refresh_token(Some(&tm)).await.unwrap();

    assert_eq!(*captured.lock().unwrap(), "callback_token");
    std::fs::remove_file(&token_file).ok();
}

/// Build a base64 token whose gen_ts is `seconds_ago` seconds in the past.
fn make_expired_token(seconds_ago: i64) -> String {
    use base64::Engine;
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let gen_ts_ms = now_ms - seconds_ago * 1000;
    let expire_ts_ms = gen_ts_ms + 3_600_000;
    let payload = format!(
        "{:013},{:013}some_extra_payload_data",
        gen_ts_ms, expire_ts_ms
    );
    base64::engine::general_purpose::STANDARD.encode(payload.as_bytes())
}

#[tokio::test]
async fn test_new_http_client_auto_refresh_on_creation() {
    use std::sync::{Arc, Mutex};
    let call_count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    let call_count_clone = Arc::clone(&call_count);

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"code":0,"message":"success","data":{"token":"auto_created_token"}}"#,
        ))
        .mount(&mock_server)
        .await;

    let mut config = test_config(&mock_server.uri());
    config.token = Some(make_expired_token(100)); // already expired
    config.token_refresh_duration = Some(Duration::from_secs(30));
    config.token_check_interval = Some(Duration::from_millis(100));

    // Wrap call_count into token_writer for verification
    config.token_writer = Some(std::sync::Arc::new(move |_t: String| {
        *call_count_clone.lock().unwrap() += 1;
    }));

    let client = HttpClient::new(config);
    // Poll the token until the background refresh updates it (timing-sensitive on CI).
    // Use a generous deadline so slow runners still pass; the assert below is the real check.
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        {
            let token = client.config.read().unwrap().token.clone();
            if token == Some("auto_created_token".to_string()) {
                break;
            }
        }
        if std::time::Instant::now() > deadline {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    client.close();

    // config.token should have been updated by the background refresh
    let token = client.config.read().unwrap().token.clone();
    assert_eq!(token, Some("auto_created_token".to_string()));
}

#[tokio::test]
async fn test_new_http_client_no_auto_refresh_when_zero() {
    let mock_server = MockServer::start().await;
    // If any request is made it fails the test by not returning valid data
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"code":0,"message":"success","data":{"token":"should_not_appear"}}"#,
        ))
        .mount(&mock_server)
        .await;

    let mut config = test_config(&mock_server.uri());
    config.token = Some(make_expired_token(100));
    // token_refresh_duration is None → no auto-refresh

    let _client = HttpClient::new(config);
    tokio::time::sleep(Duration::from_millis(200)).await;

    // The mock server should have received zero token-refresh requests
    // (test requests from other tests won't reach this mock server because it's unique)
    let received = mock_server.received_requests().await.unwrap();
    assert_eq!(received.len(), 0, "no refresh should have been triggered");
}

#[tokio::test]
async fn test_close_stops_background_task() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(
                r#"{"code":0,"message":"success","data":{"token":"stopped_token"}}"#,
            ),
        )
        .mount(&mock_server)
        .await;

    let mut config = test_config(&mock_server.uri());
    config.token = Some(make_expired_token(100));
    config.token_refresh_duration = Some(Duration::from_secs(30));
    config.token_check_interval = Some(Duration::from_millis(100));

    let client = HttpClient::new(config);
    tokio::time::sleep(Duration::from_millis(300)).await;
    client.close();

    // Allow a brief grace period for any in-flight HTTP request (already sent
    // before the stop signal arrived) to land at the mock server, so the
    // "after close" snapshot is stable.
    tokio::time::sleep(Duration::from_millis(200)).await;
    let count_after_close = mock_server.received_requests().await.unwrap().len();
    tokio::time::sleep(Duration::from_millis(300)).await;
    let count_final = mock_server.received_requests().await.unwrap().len();

    // No NEW refresh cycles should be triggered after close(). A single
    // in-flight request that was already sent before close() may still land,
    // so we tolerate count_final <= count_after_close + 1.
    assert!(
        count_final <= count_after_close + 1,
        "no new refresh cycles after close() (got {} -> {})",
        count_after_close,
        count_final
    );
}

// ========== extract_token_owned helper tests ==========

#[test]
fn test_extract_token_owned_object_data() {
    // data is a JSON object with token field
    let json = serde_json::json!({
        "code": 0,
        "message": "success",
        "data": {"token": "abc123"}
    });
    assert_eq!(extract_token_owned(&json), "abc123");
}

#[test]
fn test_extract_token_owned_double_encoded_string() {
    // data is a JSON string (double-encoded) containing an object with token
    let json = serde_json::json!({
        "code": 0,
        "message": "success",
        "data": r#"{"token":"xyz789"}"#
    });
    assert_eq!(extract_token_owned(&json), "xyz789");
}

#[test]
fn test_extract_token_owned_data_missing() {
    let json = serde_json::json!({"code": 0, "message": "ok"});
    assert_eq!(extract_token_owned(&json), "");
}

#[test]
fn test_extract_token_owned_token_missing() {
    let json = serde_json::json!({"code": 0, "data": {"other": "val"}});
    assert_eq!(extract_token_owned(&json), "");
}

#[test]
fn test_extract_token_owned_double_encoded_invalid_json() {
    let json = serde_json::json!({
        "code": 0,
        "data": "not valid json"
    });
    assert_eq!(extract_token_owned(&json), "");
}

#[test]
fn test_extract_token_owned_double_encoded_no_token_field() {
    let json = serde_json::json!({
        "code": 0,
        "data": r#"{"other":"val"}"#
    });
    assert_eq!(extract_token_owned(&json), "");
}

// ========== user_agent / build_common_params edge cases ==========

#[test]
fn test_user_agent_format() {
    let ua = HttpClient::user_agent();
    assert!(ua.starts_with("openapi-rust-sdk-"));
    assert!(ua.len() > "openapi-rust-sdk-".len());
}

#[tokio::test]
async fn test_with_quote_server_uses_quote_url() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"code":0,"message":"success","data":null}"#,
        ))
        .expect(1)
        .mount(&mock_server)
        .await;

    let mut config = test_config("http://this-should-not-be-used.invalid");
    config.quote_server_url = mock_server.uri();
    let client = HttpClient::with_quote_server(config);
    let result = client.execute("market_state", r#"{"market":"US"}"#).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execute_request_with_version_override() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"code":0,"message":"ok","data":null}"#),
        )
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let client = HttpClient::new(config);
    let req = ApiRequest::with_version("option_chain", r#"{"symbol":"AAPL"}"#, "3.0");
    let result = client.execute_request(&req).await;
    assert!(result.is_ok());

    let received = mock_server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    // version should be the overridden "3.0", not default "2.0"
    assert_eq!(body["version"], "3.0");
}

#[tokio::test]
async fn test_execute_request_trade_method_no_retry_on_network_error() {
    // place_order is a trade operation → should not retry on network failure
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500).set_body_string("server error"))
        .expect(1) // only 1 attempt, no retries
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let client = HttpClient::new(config);
    let req = ApiRequest::new("place_order", r#"{"account":"DU1"}"#);
    let result = client.execute_request(&req).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_query_token_double_encoded_response() {
    // Server returns data as a JSON string (double-encoded)
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"code":0,"message":"success","data":"{\"token\":\"double_encoded_token\"}"}"#,
        ))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let client = HttpClient::new(config);
    let token = client.query_token().await.unwrap();
    assert_eq!(token, "double_encoded_token");
}

#[tokio::test]
async fn test_query_token_missing_data_returns_error() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"code":0,"message":"success","data":null}"#,
        ))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let client = HttpClient::new(config);
    assert!(client.query_token().await.is_err());
}
