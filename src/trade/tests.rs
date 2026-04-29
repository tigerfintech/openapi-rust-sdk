//! TradeClient 测试模块。
//! 使用 wiremock mock HTTP 响应，验证各交易方法的请求构造和响应解析。

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
        account: "test_account".to_string(),
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

// ========== 19.1 合约查询测试 ==========

#[tokio::test]
async fn test_get_contract() {
    let server = mock_success_server(r#"{"symbol":"AAPL","secType":"STK"}"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let data = tc.get_contract("AAPL", "STK").await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_contracts() {
    let server = mock_success_server(r#"[{"symbol":"AAPL","secType":"STK"},{"symbol":"GOOG","secType":"STK"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let data = tc.get_contracts(&["AAPL", "GOOG"], "STK").await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_quote_contract() {
    let server = mock_success_server(r#"{"symbol":"AAPL","secType":"OPT"}"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let data = tc.get_quote_contract("AAPL", "OPT").await.unwrap();
    assert!(data.is_some());
}

// ========== 19.3 订单操作测试 ==========

#[tokio::test]
async fn test_place_order() {
    let server = mock_success_server(r#"{"id":12345,"orderId":1}"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let order = serde_json::json!({
        "symbol": "AAPL",
        "secType": "STK",
        "action": "BUY",
        "orderType": "LMT",
        "quantity": 100,
        "limitPrice": 150.0,
        "timeInForce": "DAY"
    });
    let data = tc.place_order(order).await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_preview_order() {
    let server = mock_success_server(r#"{"estimatedCommission":1.5}"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let order = serde_json::json!({
        "symbol": "AAPL",
        "secType": "STK",
        "action": "BUY",
        "orderType": "MKT",
        "quantity": 100
    });
    let data = tc.preview_order(order).await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_modify_order() {
    let server = mock_success_server(r#"{"id":12345}"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let order = serde_json::json!({
        "orderType": "LMT",
        "quantity": 200,
        "limitPrice": 155.0
    });
    let data = tc.modify_order(12345, order).await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_cancel_order() {
    let server = mock_success_server(r#"{"id":12345}"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let data = tc.cancel_order(12345).await.unwrap();
    assert!(data.is_some());
}

// ========== 19.5 订单查询测试 ==========

#[tokio::test]
async fn test_get_orders() {
    let server = mock_success_server(r#"[{"id":1,"symbol":"AAPL"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let data = tc.get_orders().await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_active_orders() {
    let server = mock_success_server(r#"[{"id":1,"status":"Submitted"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let data = tc.get_active_orders().await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_inactive_orders() {
    let server = mock_success_server(r#"[{"id":1,"status":"Cancelled"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let data = tc.get_inactive_orders().await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_filled_orders() {
    let server = mock_success_server(r#"[{"id":1,"status":"Filled"}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let data = tc.get_filled_orders().await.unwrap();
    assert!(data.is_some());
}

// ========== 19.7 持仓和资产查询测试 ==========

#[tokio::test]
async fn test_get_positions() {
    let server = mock_success_server(r#"[{"symbol":"AAPL","quantity":100}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let data = tc.get_positions().await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_assets() {
    let server = mock_success_server(r#"{"netLiquidation":100000.0}"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let data = tc.get_assets().await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_prime_assets() {
    let server = mock_success_server(r#"{"netLiquidation":200000.0}"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let data = tc.get_prime_assets().await.unwrap();
    assert!(data.is_some());
}

#[tokio::test]
async fn test_get_order_transactions() {
    let server = mock_success_server(r#"[{"id":12345,"filledQuantity":50}]"#).await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let data = tc.get_order_transactions(12345).await.unwrap();
    assert!(data.is_some());
}

// ========== 错误处理测试 ==========

#[tokio::test]
async fn test_trade_client_api_error() {
    let server = mock_error_server(1100, "交易操作失败").await;
    let config = test_config(&server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let result = tc.get_orders().await;
    assert!(result.is_err());
}

// ========== 请求参数验证测试 ==========

#[tokio::test]
async fn test_place_order_sends_correct_method() {
    let mock_server = MockServer::start().await;
    let response_body = r#"{"code":0,"message":"success","data":{"id":1},"timestamp":1700000000}"#;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let order = serde_json::json!({"symbol": "AAPL", "action": "BUY"});
    let _ = tc.place_order(order).await;

    let received = mock_server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);
    let req_body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req_body["method"].as_str().unwrap(), "place_order");
}

#[tokio::test]
async fn test_place_order_includes_account() {
    let mock_server = MockServer::start().await;
    let response_body = r#"{"code":0,"message":"success","data":{"id":1},"timestamp":1700000000}"#;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let order = serde_json::json!({"symbol": "AAPL", "action": "BUY"});
    let _ = tc.place_order(order).await;

    // 验证 biz_content 中包含 account 字段
    let received = mock_server.received_requests().await.unwrap();
    let req_body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    let biz: serde_json::Value = serde_json::from_str(req_body["biz_content"].as_str().unwrap()).unwrap();
    assert_eq!(biz["account"].as_str().unwrap(), "test_account");
}

#[tokio::test]
async fn test_cancel_order_sends_correct_params() {
    let mock_server = MockServer::start().await;
    let response_body = r#"{"code":0,"message":"success","data":{"id":12345},"timestamp":1700000000}"#;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let _ = tc.cancel_order(12345).await;

    let received = mock_server.received_requests().await.unwrap();
    let req_body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req_body["method"].as_str().unwrap(), "cancel_order");
    let biz: serde_json::Value = serde_json::from_str(req_body["biz_content"].as_str().unwrap()).unwrap();
    assert_eq!(biz["account"].as_str().unwrap(), "test_account");
    assert_eq!(biz["id"].as_i64().unwrap(), 12345);
}

#[tokio::test]
async fn test_modify_order_includes_id_and_account() {
    let mock_server = MockServer::start().await;
    let response_body = r#"{"code":0,"message":"success","data":{"id":12345},"timestamp":1700000000}"#;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let http_client = HttpClient::new(config);
    let tc = TradeClient::new(&http_client, "test_account");

    let order = serde_json::json!({"limitPrice": 155.0});
    let _ = tc.modify_order(12345, order).await;

    let received = mock_server.received_requests().await.unwrap();
    let req_body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req_body["method"].as_str().unwrap(), "modify_order");
    let biz: serde_json::Value = serde_json::from_str(req_body["biz_content"].as_str().unwrap()).unwrap();
    assert_eq!(biz["account"].as_str().unwrap(), "test_account");
    assert_eq!(biz["id"].as_i64().unwrap(), 12345);
    assert_eq!(biz["limitPrice"].as_f64().unwrap(), 155.0);
}
