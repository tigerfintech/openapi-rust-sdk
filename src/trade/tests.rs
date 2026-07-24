//! TradeClient 测试模块。
//! 使用 wiremock 验证请求使用 snake_case，响应被解析为强类型。

use super::*;
use std::sync::OnceLock;
use std::time::Duration;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use rsa::RsaPrivateKey;

use crate::client::http_client::HttpClient;
use crate::config::client_config::ClientConfig;
use crate::model::order::limit_order;
use crate::model::trade_requests::{
    AssetsRequest, OrderTransactionsRequest, OrdersRequest, PositionsRequest,
};

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

fn test_config(server_url: &str) -> ClientConfig {
    ClientConfig {
        tiger_id: "test_tiger_id".to_string(),
        private_key: cached_test_private_key().to_string(),
        account: "test_account".to_string(),
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

fn biz_of(req: &wiremock::Request) -> serde_json::Value {
    let v: serde_json::Value = serde_json::from_slice(&req.body).unwrap();
    let s = v["biz_content"].as_str().unwrap();
    serde_json::from_str(s).unwrap()
}

// ========== typed response 测试 ==========

#[tokio::test]
async fn test_get_contract_unwraps_items_typed() {
    let server = mock_success_server(
        r#"{"items":[{"symbol":"AAPL","secType":"STK","contractId":12345,"primaryExchange":"NASDAQ"}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let contracts = tc.get_contract("AAPL", "STK").await.unwrap();
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0].symbol, "AAPL");
    assert_eq!(contracts[0].sec_type, "STK");
    assert_eq!(contracts[0].contract_id, Some(12345));
}

#[tokio::test]
async fn test_get_contracts_unwraps_items_typed() {
    let server = mock_success_server(
        r#"{"items":[{"symbol":"AAPL","secType":"STK"},{"symbol":"GOOG","secType":"STK"}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let cs = tc.get_contracts(&["AAPL", "GOOG"], "STK").await.unwrap();
    assert_eq!(cs.len(), 2);
}

#[tokio::test]
async fn test_get_quote_contract_unwraps_items() {
    let server =
        mock_success_server(r#"{"items":[{"symbol":"AAPL","secType":"OPT","expiry":"20260619"}]}"#)
            .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let cs = tc
        .get_quote_contract("AAPL", "OPT", "20260619")
        .await
        .unwrap();
    assert_eq!(cs.len(), 1);
    assert_eq!(cs[0].sec_type, "OPT");
}

#[tokio::test]
async fn test_preview_order_typed() {
    let server = mock_success_server(
        r#"{"isPass":true,"commission":0.5,"commissionCurrency":"USD","initMargin":50.0}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let req = limit_order("test_account", "AAPL", "STK", "BUY", 1, 1.00);
    let p = tc.preview_order(req).await.unwrap();
    let p = p.expect("preview should return data");
    assert!(p.is_pass);
    assert_eq!(p.commission, 0.5);
}

#[tokio::test]
async fn test_place_order_typed_returns_id_and_order_id() {
    let server = mock_success_server(r#"{"id":42519413060422656,"order_id":143}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let req = limit_order("test_account", "AAPL", "STK", "BUY", 1, 1.00);
    let r = tc.place_order(req).await.unwrap().expect("placed");
    assert_eq!(r.id, 42519413060422656);
    assert_eq!(r.order_id, 143);
}

#[tokio::test]
async fn test_cancel_order_typed() {
    let server = mock_success_server(r#"{"id":12345}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let r = tc.cancel_order(12345).await.unwrap().expect("canceled");
    assert_eq!(r.id, 12345);
}

#[tokio::test]
async fn test_get_orders_unwraps_items_typed() {
    let server = mock_success_server(
        r#"{"items":[{"id":1,"orderId":100,"symbol":"AAPL","secType":"STK","status":"Submitted","totalQuantity":10,"limitPrice":150.5}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let orders = tc.get_orders(OrdersRequest::default()).await.unwrap();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].id, 1);
    assert_eq!(orders[0].order_id, 100);
    assert_eq!(orders[0].sec_type, "STK");
    assert_eq!(orders[0].total_quantity, 10);
}

#[tokio::test]
async fn test_get_filled_orders_unwraps_items_typed() {
    let server =
        mock_success_server(r#"{"items":[{"id":1,"status":"Filled","filledQuantity":50}]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let req = OrdersRequest {
        start_date: Some(0),
        end_date: Some(0),
        ..Default::default()
    };
    let orders = tc.get_filled_orders(req).await.unwrap();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].status, "Filled");
}

#[tokio::test]
async fn test_get_positions_unwraps_items_typed() {
    let server = mock_success_server(
        r#"{"items":[{"symbol":"AAPL","secType":"STK","position":100,"averageCost":150.0,"marketValue":15500.0}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let ps = tc.get_positions(PositionsRequest::default()).await.unwrap();
    assert_eq!(ps.len(), 1);
    assert_eq!(ps[0].symbol, Some("AAPL".to_string()));
    assert_eq!(ps[0].position, Some(100));
}

#[tokio::test]
async fn test_get_assets_unwraps_items_typed() {
    let server = mock_success_server(
        r#"{"items":[{"account":"DU123","currency":"USD","buyingPower":10000.0,"netLiquidation":20000.0}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let assets = tc.get_assets(AssetsRequest::default()).await.unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].account, "DU123");
    assert_eq!(assets[0].buying_power, 10000.0);
}

#[tokio::test]
async fn test_get_prime_assets_typed_no_items_wrap() {
    let server = mock_success_server(
        r#"{"accountId":"acc1","updateTimestamp":1700000000,"segments":[{"capability":"MARGIN","category":"S","currency":"USD","buyingPower":10000.0}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let pa = tc
        .get_prime_assets(AssetsRequest::default())
        .await
        .unwrap()
        .expect("prime_assets");
    assert_eq!(pa.account_id, "acc1");
    assert_eq!(pa.segments.len(), 1);
    assert_eq!(pa.segments[0].buying_power, 10000.0);
}

#[tokio::test]
async fn test_get_order_transactions_unwraps_items_typed() {
    let server = mock_success_server(
        r#"{"items":[{"id":1,"orderId":2,"symbol":"AAPL","secType":"STK","price":150.0,"filledQuantity":50}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let req = OrderTransactionsRequest {
        order_id: Some(2),
        symbol: Some("AAPL".to_string()),
        sec_type: Some("STK".to_string()),
        ..Default::default()
    };
    let txs = tc.get_order_transactions(req).await.unwrap();
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].order_id, 2);
    assert_eq!(txs[0].filled_quantity, 50);
}

// ========== 请求参数 snake_case 测试 ==========

#[tokio::test]
async fn test_get_contract_wire_snake_case() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc.get_contract("AAPL", "STK").await;

    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "contract");
    let biz = biz_of(&received[0]);
    assert_eq!(biz["account"].as_str().unwrap(), "test_account");
    assert_eq!(biz["sec_type"].as_str().unwrap(), "STK");
    assert!(biz.get("secType").is_none());
}

#[tokio::test]
async fn test_get_quote_contract_wire_has_symbols_and_expiry() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc.get_quote_contract("AAPL", "OPT", "20260619").await;

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert_eq!(biz["symbols"][0].as_str().unwrap(), "AAPL");
    assert_eq!(biz["expiry"].as_str().unwrap(), "20260619");
    assert_eq!(biz["sec_type"].as_str().unwrap(), "OPT");
}

#[tokio::test]
async fn test_place_order_wire_snake_case() {
    let server = mock_success_server(r#"{"id":1,"order_id":1}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let req = limit_order("test_account", "AAPL", "STK", "BUY", 1, 1.0);
    let _ = tc.place_order(req).await;

    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "place_order");
    let biz = biz_of(&received[0]);
    assert_eq!(biz["account"].as_str().unwrap(), "test_account");
    assert_eq!(biz["sec_type"].as_str().unwrap(), "STK");
    assert_eq!(biz["order_type"].as_str().unwrap(), "LMT");
    assert_eq!(biz["total_quantity"].as_i64().unwrap(), 1);
    assert_eq!(biz["limit_price"].as_f64().unwrap(), 1.0);
    assert_eq!(biz["time_in_force"].as_str().unwrap(), "DAY");
    // no camelCase
    assert!(biz.get("secType").is_none());
    assert!(biz.get("orderType").is_none());
    assert!(biz.get("totalQuantity").is_none());
    assert!(biz.get("limitPrice").is_none());
    assert!(biz.get("timeInForce").is_none());
}

#[tokio::test]
async fn test_modify_order_wire_includes_id_and_account() {
    let server = mock_success_server(r#"{"id":12345}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let mut req = limit_order("test_account", "AAPL", "STK", "BUY", 1, 1.5);
    req.limit_price = Some(1.5);
    let _ = tc.modify_order(12345, req).await;

    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "modify_order");
    let biz = biz_of(&received[0]);
    assert_eq!(biz["id"].as_i64().unwrap(), 12345);
    assert_eq!(biz["account"].as_str().unwrap(), "test_account");
    assert_eq!(biz["limit_price"].as_f64().unwrap(), 1.5);
}

#[tokio::test]
async fn test_cancel_order_wire_has_account_and_id() {
    let server = mock_success_server(r#"{"id":12345}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc.cancel_order(12345).await;

    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "cancel_order");
    let biz = biz_of(&received[0]);
    assert_eq!(biz["account"].as_str().unwrap(), "test_account");
    assert_eq!(biz["id"].as_i64().unwrap(), 12345);
}

#[tokio::test]
async fn test_get_filled_orders_wire_snake_case_dates() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let req = OrdersRequest {
        start_date: Some(1700000000000),
        end_date: Some(1710000000000),
        ..Default::default()
    };
    let _ = tc.get_filled_orders(req).await;

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert_eq!(biz["start_date"].as_i64().unwrap(), 1700000000000);
    assert_eq!(biz["end_date"].as_i64().unwrap(), 1710000000000);
    assert!(biz.get("startDate").is_none());
}

#[tokio::test]
async fn test_get_order_transactions_wire_snake_case() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");

    let req = OrderTransactionsRequest {
        order_id: Some(12345),
        symbol: Some("AAPL".to_string()),
        sec_type: Some("STK".to_string()),
        ..Default::default()
    };
    let _ = tc.get_order_transactions(req).await;

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert_eq!(biz["order_id"].as_i64().unwrap(), 12345);
    assert_eq!(biz["symbol"].as_str().unwrap(), "AAPL");
    assert_eq!(biz["sec_type"].as_str().unwrap(), "STK");
    assert!(biz.get("orderId").is_none());
}

#[tokio::test]
async fn test_trade_api_error() {
    let server = mock_error_server(1100, "交易操作失败").await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    assert!(tc.get_orders(OrdersRequest::default()).await.is_err());
}

// ========== call_* 低级接口直接调用测试 ==========

#[tokio::test]
async fn test_call_optional_returns_some() {
    let server = mock_success_server(
        r#"{"isPass":true,"commission":1.0,"commissionCurrency":"USD","initMargin":0.0}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let result: Option<crate::model::trade::PreviewResult> = tc
        .call_optional(
            "preview_order",
            serde_json::json!({"account": "test_account"}),
        )
        .await
        .unwrap();
    let r = result.expect("should be Some");
    assert!(r.is_pass);
    assert_eq!(r.commission, 1.0);
}

#[tokio::test]
async fn test_call_optional_returns_none_on_null_data() {
    let server = mock_success_server("null").await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let result: Option<crate::model::trade::PreviewResult> = tc
        .call_optional("preview_order", serde_json::json!({}))
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_call_into_items_unwraps_items_key() {
    let server = mock_success_server(r#"{"items":[{"symbol":"TSLA","secType":"STK"}]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let items: Vec<crate::model::contract::Contract> = tc
        .call_into_items(
            "contract",
            serde_json::json!({"account": "test_account", "symbol": "TSLA", "sec_type": "STK"}),
        )
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].symbol, "TSLA");
}

#[tokio::test]
async fn test_call_into_items_empty_on_null_data() {
    let server = mock_success_server("null").await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let items: Vec<crate::model::contract::Contract> = tc
        .call_into_items("contract", serde_json::json!({}))
        .await
        .unwrap();
    assert!(items.is_empty());
}

#[tokio::test]
async fn test_call_into_items_falls_back_to_array() {
    let server = mock_success_server(r#"[{"symbol":"AAPL","secType":"STK"}]"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let items: Vec<crate::model::contract::Contract> = tc
        .call_into_items("contract", serde_json::json!({}))
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].symbol, "AAPL");
}
