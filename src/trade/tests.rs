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

// ========== 缺失方法的测试（Batch：wire method + typed 解析） ==========

use crate::model::trade_requests::{
    AggregateAssetsRequest, AnalyticsAssetRequest, DerivativeContractsRequest,
    EstimateTradableQuantityRequest, ForexOrderRequest, FundDetailsRequest, FundingHistoryRequest,
    GetOrderRequest, ManagedAccountsRequest, OptionExerciseCancelRequest,
    OptionExerciseCheckRequest, OptionExercisePositionRequest, OptionExerciseRecordsRequest,
    OptionExerciseSubmitRequest, PositionTransferDetailRequest,
    PositionTransferExternalRecordsRequest, PositionTransferRecordsRequest,
    PositionTransferRequest, SegmentFundRequest,
};

// --- 1. get_active_orders ---

#[tokio::test]
async fn test_get_active_orders_wire_method() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc.get_active_orders(OrdersRequest::default()).await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "active_orders");
}

#[tokio::test]
async fn test_get_active_orders_typed() {
    let server = mock_success_server(
        r#"{"items":[{"id":10,"orderId":200,"symbol":"TSLA","secType":"STK","status":"Submitted","totalQuantity":5}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let orders = tc
        .get_active_orders(OrdersRequest::default())
        .await
        .unwrap();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].id, 10);
    assert_eq!(orders[0].status, "Submitted");
}

// --- 2. get_inactive_orders ---

#[tokio::test]
async fn test_get_inactive_orders_wire_method() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc.get_inactive_orders(OrdersRequest::default()).await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "inactive_orders");
}

#[tokio::test]
async fn test_get_inactive_orders_typed() {
    let server = mock_success_server(
        r#"{"items":[{"id":11,"orderId":201,"symbol":"GOOG","secType":"STK","status":"Cancelled"}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let orders = tc
        .get_inactive_orders(OrdersRequest::default())
        .await
        .unwrap();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].status, "Cancelled");
}

// --- 3. get_order (single) ---

#[tokio::test]
async fn test_get_order_wire_method() {
    let server = mock_success_server(
        r#"{"id":99,"orderId":500,"symbol":"AAPL","secType":"STK","status":"Filled"}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let req = GetOrderRequest {
        id: Some(99),
        ..Default::default()
    };
    let _ = tc.get_order(req).await;
    let received = server.received_requests().await.unwrap();
    let r: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(r["method"].as_str().unwrap(), "order_no");
}

#[tokio::test]
async fn test_get_order_returns_none_when_both_ids_zero() {
    let server = mock_success_server(r#"null"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let req = GetOrderRequest {
        id: Some(0),
        order_id: Some(0),
        ..Default::default()
    };
    let result = tc.get_order(req).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_order_returns_some_when_id_set() {
    let server = mock_success_server(
        r#"{"id":42,"orderId":7,"symbol":"MSFT","secType":"STK","status":"Filled","totalQuantity":10}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let req = GetOrderRequest {
        id: Some(42),
        ..Default::default()
    };
    let result = tc.get_order(req).await.unwrap();
    let order = result.expect("should be Some");
    assert_eq!(order.id, 42);
    assert_eq!(order.symbol, "MSFT");
}

// --- 4. get_managed_accounts ---

#[tokio::test]
async fn test_get_managed_accounts_wire_method() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .get_managed_accounts(ManagedAccountsRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "accounts");
}

#[tokio::test]
async fn test_get_managed_accounts_typed() {
    let server = mock_success_server(
        r#"{"items":[{"account":"DU001","accountType":"MARGIN","capability":"RegTMargin","status":"Active"}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let accs = tc
        .get_managed_accounts(ManagedAccountsRequest::default())
        .await
        .unwrap();
    assert_eq!(accs.len(), 1);
    assert_eq!(accs[0].account, "DU001");
    assert_eq!(accs[0].status, "Active");
}

// --- 5. get_derivative_contracts ---

#[tokio::test]
async fn test_get_derivative_contracts_wire_method() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .get_derivative_contracts(DerivativeContractsRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "quote_contract");
}

// --- 6. get_analytics_asset ---

#[tokio::test]
async fn test_get_analytics_asset_wire_method() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .get_analytics_asset(AnalyticsAssetRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "analytics_asset");
}

#[tokio::test]
async fn test_get_analytics_asset_typed() {
    let server = mock_success_server(
        r#"{"items":[{"date":"2025-01-01","holdingValue":50000.0,"cashBalance":10000.0,"pnl":500.0,"currency":"USD","segType":"S"}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let items = tc
        .get_analytics_asset(AnalyticsAssetRequest::default())
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].date, "2025-01-01");
    assert_eq!(items[0].holding_value, 50000.0);
}

// --- 7. get_aggregate_assets ---

#[tokio::test]
async fn test_get_aggregate_assets_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .get_aggregate_assets(AggregateAssetsRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "aggregate_assets");
}

#[tokio::test]
async fn test_get_aggregate_assets_typed() {
    let server = mock_success_server(
        r#"{"accountId":"acc1","netLiquidation":100000.0,"grossPositionValue":80000.0,"cashBalance":20000.0,"baseCurrency":"USD"}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let result = tc
        .get_aggregate_assets(AggregateAssetsRequest::default())
        .await
        .unwrap();
    let aa = result.expect("should be Some");
    assert_eq!(aa.account_id, "acc1");
    assert_eq!(aa.net_liquidation, 100000.0);
}

// --- 8. get_estimate_tradable_quantity ---

#[tokio::test]
async fn test_get_estimate_tradable_quantity_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .get_estimate_tradable_quantity(EstimateTradableQuantityRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(
        req["method"].as_str().unwrap(),
        "estimate_tradable_quantity"
    );
}

#[tokio::test]
async fn test_get_estimate_tradable_quantity_typed() {
    let server = mock_success_server(
        r#"{"tradableQuantity":100.0,"maxCashBuyQuantity":80.0,"maxMarginBuyQuantity":150.0,"currency":"USD"}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let result = tc
        .get_estimate_tradable_quantity(EstimateTradableQuantityRequest::default())
        .await
        .unwrap();
    let etq = result.expect("should be Some");
    assert_eq!(etq.tradable_quantity, 100.0);
    assert_eq!(etq.currency, "USD");
}

// --- 9. place_forex_order ---

#[tokio::test]
async fn test_place_forex_order_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc.place_forex_order(ForexOrderRequest::default()).await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "place_forex_order");
}

#[tokio::test]
async fn test_place_forex_order_typed() {
    // Server returns `id` as a JSON number (matches PlaceOrderResult.id and
    // the actual wire contract observed on the Go SDK integ). Earlier the
    // mock returned a string, hiding a type mismatch that broke real
    // forex responses.
    let server = mock_success_server(
        r#"{"id":12345,"status":"Submitted","sourceCurrency":"USD","targetCurrency":"HKD","sourceAmount":1000.0,"rate":7.8}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let result = tc
        .place_forex_order(ForexOrderRequest {
            source_currency: Some("USD".into()),
            target_currency: Some("HKD".into()),
            source_amount: Some(1000.0),
            ..Default::default()
        })
        .await
        .unwrap();
    let fx = result.expect("should be Some");
    assert_eq!(fx.id, 12345);
    assert_eq!(fx.source_currency, "USD");
}

// --- 10. get_segment_fund_available ---

#[tokio::test]
async fn test_get_segment_fund_available_wire_method() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .get_segment_fund_available(SegmentFundRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "segment_fund_available");
}

#[tokio::test]
async fn test_get_segment_fund_available_typed() {
    let server =
        mock_success_server(r#"{"items":[{"fromSegment":"S","currency":"USD","amount":5000.0}]}"#)
            .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let items = tc
        .get_segment_fund_available(SegmentFundRequest::default())
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].from_segment, "S");
    assert_eq!(items[0].amount, 5000.0);
}

// --- 11. get_segment_fund_history ---

#[tokio::test]
async fn test_get_segment_fund_history_wire_method() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .get_segment_fund_history(SegmentFundRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "segment_fund_history");
}

// --- 12. transfer_segment_fund ---

#[tokio::test]
async fn test_transfer_segment_fund_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .transfer_segment_fund(SegmentFundRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "transfer_segment_fund");
}

#[tokio::test]
async fn test_transfer_segment_fund_typed() {
    let server = mock_success_server(
        r#"{"id":123,"fromSegment":"S","toSegment":"C","currency":"USD","amount":1000.0,"status":"Pending"}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let result = tc
        .transfer_segment_fund(SegmentFundRequest {
            from_segment: Some("S".into()),
            to_segment: Some("C".into()),
            currency: Some("USD".into()),
            amount: Some(1000.0),
            ..Default::default()
        })
        .await
        .unwrap();
    let sf = result.expect("should be Some");
    assert_eq!(sf.from_segment, "S");
    assert_eq!(sf.status, "Pending");
}

// --- 13. cancel_segment_fund ---

#[tokio::test]
async fn test_cancel_segment_fund_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc.cancel_segment_fund(SegmentFundRequest::default()).await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "cancel_segment_fund");
}

// --- 14. get_fund_details ---

#[tokio::test]
async fn test_get_fund_details_wire_method() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc.get_fund_details(FundDetailsRequest::default()).await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "fund_details");
}

#[tokio::test]
async fn test_get_fund_details_typed() {
    let server = mock_success_server(
        r#"{"items":[{"id":"FD001","account":"DU123","segType":"S","fundType":"deposit","currency":"USD","amount":10000.0,"balance":50000.0}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let items = tc
        .get_fund_details(FundDetailsRequest::default())
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, "FD001");
    assert_eq!(items[0].amount, 10000.0);
}

// --- 15. get_funding_history ---

#[tokio::test]
async fn test_get_funding_history_wire_method() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .get_funding_history(FundingHistoryRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "transfer_fund");
}

#[tokio::test]
async fn test_get_funding_history_typed() {
    let server = mock_success_server(
        r#"{"items":[{"id":1,"refId":"REF001","type":1,"typeDesc":"Deposit","currency":"USD","amount":5000.0,"status":"Completed","completedStatus":true}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let items = tc
        .get_funding_history(FundingHistoryRequest::default())
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].ref_id, "REF001");
    assert!(items[0].completed_status);
}

// --- 16. transfer_position ---

#[tokio::test]
async fn test_transfer_position_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .transfer_position(PositionTransferRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "position_transfer");
}

#[tokio::test]
async fn test_transfer_position_typed() {
    let server = mock_success_server(
        r#"{"id":"PT001","fromAccount":"DU001","toAccount":"DU002","market":"US","status":"Pending","submitTime":1700000000}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let result = tc
        .transfer_position(PositionTransferRequest {
            from_account: Some("DU001".into()),
            to_account: Some("DU002".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    let pt = result.expect("should be Some");
    assert_eq!(pt.id, "PT001");
    assert_eq!(pt.status, "Pending");
}

// --- 17. get_position_transfer_records ---

#[tokio::test]
async fn test_get_position_transfer_records_wire_method() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .get_position_transfer_records(PositionTransferRecordsRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "position_transfer_records");
}

// --- 18. get_position_transfer_detail ---

#[tokio::test]
async fn test_get_position_transfer_detail_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .get_position_transfer_detail(PositionTransferDetailRequest {
            id: Some("PT001".into()),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "position_transfer_detail");
}

#[tokio::test]
async fn test_get_position_transfer_detail_typed() {
    let server = mock_success_server(
        r#"{"id":"PT001","fromAccount":"DU001","toAccount":"DU002","market":"US","status":"Completed","submitTime":1700000000,"updateTime":1700001000,"transfers":[]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let result = tc
        .get_position_transfer_detail(PositionTransferDetailRequest {
            id: Some("PT001".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    let detail = result.expect("should be Some");
    assert_eq!(detail.id, "PT001");
    assert_eq!(detail.status, "Completed");
}

// --- 19. get_position_transfer_external_records ---

#[tokio::test]
async fn test_get_position_transfer_external_records_wire_method() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .get_position_transfer_external_records(PositionTransferExternalRecordsRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(
        req["method"].as_str().unwrap(),
        "position_transfer_external_records"
    );
}

// --- 20. option_exercise_check ---

#[tokio::test]
async fn test_option_exercise_check_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .option_exercise_check(OptionExerciseCheckRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "option_exercise_check");
}

#[tokio::test]
async fn test_option_exercise_check_typed() {
    let server = mock_success_server(
        r#"{"availableQuantity":10.0,"position":10.0,"stkPosition":100.0,"stkPositionChange":1000.0,"stkPositionBefore":100.0,"stkPositionAfter":1100.0,"symbol":"AAPL"}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let result = tc
        .option_exercise_check(OptionExerciseCheckRequest {
            contract_id: Some(12345),
            exercise_type: Some("Exercise".into()),
            quantity: Some(10.0),
            ..Default::default()
        })
        .await
        .unwrap();
    let check = result.expect("should be Some");
    assert_eq!(check.available_quantity, 10.0);
    assert_eq!(check.symbol, "AAPL");
}

// --- 21. get_option_exercise_positions ---

#[tokio::test]
async fn test_get_option_exercise_positions_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .get_option_exercise_positions(OptionExercisePositionRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "option_exercise_position");
}

#[tokio::test]
async fn test_get_option_exercise_positions_typed() {
    let server = mock_success_server(
        r#"{"pageNum":1,"pageSize":20,"itemCount":1,"pageCount":1,"items":[{"contractId":123,"symbol":"AAPL 240119C00150000","stkSymbol":"AAPL","expireDate":"2024-01-19","strike":"150","callPut":"CALL","market":"US","accountId":1,"position":5.0,"availableQuantity":5.0}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let result = tc
        .get_option_exercise_positions(OptionExercisePositionRequest {
            exercise_type: Some("Exercise".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    let page = result.expect("should be Some");
    assert_eq!(page.item_count, 1);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].stk_symbol, "AAPL");
}

// --- 22. submit_option_exercise ---

#[tokio::test]
async fn test_submit_option_exercise_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .submit_option_exercise(OptionExerciseSubmitRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "option_exercise_submit");
}

#[tokio::test]
async fn test_submit_option_exercise_typed() {
    let server = mock_success_server(r#"true"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let result = tc
        .submit_option_exercise(OptionExerciseSubmitRequest {
            contract_id: Some(12345),
            exercise_type: Some("Exercise".into()),
            quantity: Some(5.0),
            executing_date: Some("2024-01-19".into()),
            is_force: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(result, Some(true));
}

// --- 23. get_option_exercise_records ---

#[tokio::test]
async fn test_get_option_exercise_records_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .get_option_exercise_records(OptionExerciseRecordsRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "option_exercise_record");
}

#[tokio::test]
async fn test_get_option_exercise_records_typed() {
    let server = mock_success_server(
        r#"{"pageNum":1,"pageSize":20,"itemCount":1,"pageCount":1,"items":[{"id":1,"contractId":123,"symbol":"AAPL 240119C00150000","stkSymbol":"AAPL","expireDate":"2024-01-19","strike":"150","callPut":"CALL","type":"Exercise","requestQuantity":5.0,"quantity":5.0,"status":"Success","executingDate":"2024-01-19","itmRate":0,"isForce":true,"reason":"","accountId":1}]}"#,
    )
    .await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let result = tc
        .get_option_exercise_records(OptionExerciseRecordsRequest::default())
        .await
        .unwrap();
    let page = result.expect("should be Some");
    assert_eq!(page.item_count, 1);
    assert_eq!(page.items[0].status, "Success");
}

// --- 24. cancel_option_exercise ---

#[tokio::test]
async fn test_cancel_option_exercise_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let _ = tc
        .cancel_option_exercise(OptionExerciseCancelRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "option_exercise_cancel");
}

#[tokio::test]
async fn test_cancel_option_exercise_typed() {
    let server = mock_success_server(r#"true"#).await;
    let tc = TradeClient::new(HttpClient::new(test_config(&server.uri())), "test_account");
    let result = tc
        .cancel_option_exercise(OptionExerciseCancelRequest {
            id: Some(1),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(result, Some(true));
}
