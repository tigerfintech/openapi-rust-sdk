//! QuoteClient 测试模块。
//! 使用 wiremock mock HTTP 响应，验证各行情方法的请求构造和响应解析。

use super::*;
use std::time::Duration;
use std::sync::OnceLock;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::method;
use rsa::RsaPrivateKey;
use rsa::pkcs8::EncodePrivateKey;
use rsa::pkcs8::LineEnding;
use crate::config::client_config::ClientConfig;
use crate::client::http_client::HttpClient;

/// 缓存测试用 RSA 私钥
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
        tiger_public_key: "".to_string(),
    }
}

/// 创建返回成功响应的 mock 服务器
async fn mock_success_server(data: &str) -> MockServer {
    let mock_server = MockServer::start().await;
    let response_body = format!(
        r#"{{"code":0,"message":"success","data":{},"timestamp":1700000000}}"#,
        data
    );
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;
    mock_server
}

/// 创建返回错误响应的 mock 服务器
async fn mock_error_server(code: i32, message: &str) -> MockServer {
    let mock_server = MockServer::start().await;
    let response_body = format!(
        r#"{{"code":{},"message":"{}","data":null,"timestamp":1700000000}}"#,
        code, message
    );
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;
    mock_server
}

// ========== 18.1 基础行情测试 ==========

#[tokio::test]
async fn test_get_market_state() {
    let server = mock_success_server(r#"[{"market":"US","status":"Trading"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_market_state("US").await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_brief() {
    let server = mock_success_server(r#"[{"symbol":"AAPL","latestPrice":150.0}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_brief(&["AAPL"]).await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_kline() {
    let server = mock_success_server(r#"[{"symbol":"AAPL","period":"day"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_kline("AAPL", "day").await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_timeline() {
    let server = mock_success_server(r#"[{"symbol":"AAPL"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_timeline(&["AAPL"]).await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_trade_tick() {
    let server = mock_success_server(r#"[{"symbol":"AAPL"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_trade_tick(&["AAPL"]).await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_quote_depth() {
    let server = mock_success_server(r#"{"symbol":"AAPL"}"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_quote_depth("AAPL").await.unwrap();
    assert!(data.is_some());
}

// ========== 18.3 期权行情测试 ==========

#[tokio::test]
async fn test_get_option_expiration() {
    let server = mock_success_server(r#"["2024-01-19","2024-02-16"]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_option_expiration("AAPL").await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_option_chain() {
    let server = mock_success_server(r#"[{"symbol":"AAPL","expiry":"2024-01-19"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_option_chain("AAPL", "2024-01-19").await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_option_brief() {
    let server = mock_success_server(r#"[{"identifier":"AAPL 240119C00150000"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_option_brief(&["AAPL 240119C00150000"]).await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_option_kline() {
    let server = mock_success_server(r#"[{"identifier":"AAPL 240119C00150000","period":"day"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_option_kline("AAPL 240119C00150000", "day").await.unwrap();
    assert!(data.is_some());
}

// ========== 18.5 期货行情测试 ==========

#[tokio::test]
async fn test_get_future_exchange() {
    let server = mock_success_server(r#"["CME","NYMEX"]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_future_exchange().await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_future_contracts() {
    let server = mock_success_server(r#"[{"symbol":"ES","exchange":"CME"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_future_contracts("CME").await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_future_real_time_quote() {
    let server = mock_success_server(r#"[{"symbol":"ES2312"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_future_real_time_quote(&["ES2312"]).await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_future_kline() {
    let server = mock_success_server(r#"[{"symbol":"ES2312","period":"day"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_future_kline("ES2312", "day").await.unwrap();
    assert!(data.is_some());
}

// ========== 18.7 基本面和资金流向测试 ==========

#[tokio::test]
async fn test_get_financial_daily() {
    let server = mock_success_server(r#"[{"symbol":"AAPL"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_financial_daily("AAPL").await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_financial_report() {
    let server = mock_success_server(r#"[{"symbol":"AAPL"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_financial_report("AAPL").await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_corporate_action() {
    let server = mock_success_server(r#"[{"symbol":"AAPL"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_corporate_action("AAPL").await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_capital_flow() {
    let server = mock_success_server(r#"{"symbol":"AAPL"}"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_capital_flow("AAPL").await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_capital_distribution() {
    let server = mock_success_server(r#"{"symbol":"AAPL"}"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.get_capital_distribution("AAPL").await.unwrap();
    assert!(data.is_some());
}

// ========== 18.9 选股器和行情权限测试 ==========

#[tokio::test]
async fn test_market_scanner() {
    let server = mock_success_server(r#"[{"symbol":"AAPL"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let params = serde_json::json!({"market": "US", "scanType": "TOP_GAINERS"});
    let data = qc.market_scanner(params).await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_grab_quote_permission() {
    let server = mock_success_server(r#"[{"name":"usStockQuote","expireAt":1700000000}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let data = qc.grab_quote_permission().await.unwrap();
    assert!(data.is_some());
}

// ========== 错误处理测试 ==========

#[tokio::test]
async fn test_quote_client_api_error() {
    let server = mock_error_server(2100, "行情查询失败").await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let result = qc.get_market_state("US").await;
    assert!(result.is_err());
}

// ========== 请求参数验证测试 ==========

#[tokio::test]
async fn test_get_market_state_sends_correct_method() {
    let mock_server = MockServer::start().await;
    let response_body = r#"{"code":0,"message":"success","data":{"market":"US"},"timestamp":1700000000}"#;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let _ = qc.get_market_state("US").await;

    // 验证请求已发送且 method 字段正确
    let received = mock_server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);
    let req_body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req_body["method"].as_str().unwrap(), "market_state");
}

#[tokio::test]
async fn test_get_kline_sends_correct_biz_content() {
    let mock_server = MockServer::start().await;
    let response_body = r#"{"code":0,"message":"success","data":null,"timestamp":1700000000}"#;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let http_client = HttpClient::new(config);
    let qc = QuoteClient::new(&http_client);

    let _ = qc.get_kline("AAPL", "day").await;

    let received = mock_server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);
    let req_body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req_body["method"].as_str().unwrap(), "kline");
    // 验证 biz_content 包含正确的业务参数
    let biz: serde_json::Value = serde_json::from_str(req_body["biz_content"].as_str().unwrap()).unwrap();
    assert_eq!(biz["symbols"].as_array().unwrap()[0].as_str().unwrap(), "AAPL");
    assert_eq!(biz["period"].as_str().unwrap(), "day");
}
