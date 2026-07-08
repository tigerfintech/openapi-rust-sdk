//! QuoteClient 测试模块。
//! 使用 wiremock mock HTTP 响应，验证各行情方法的请求构造（snake_case）和响应解析（typed）。

use super::*;
use std::sync::OnceLock;
use std::time::Duration;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use rsa::RsaPrivateKey;

use crate::client::http_client::HttpClient;
use crate::config::client_config::ClientConfig;
use crate::model::quote::{
    CorporateActionRequest, FinancialDailyRequest, FinancialReportRequest,
    MarketScannerRequest,
};
use crate::model::quote_requests::{
    BriefRequest, QuoteDepthRequest, FutureRealTimeQuoteRequest, FutureKlineRequest, KlineRequest,
    OptionChainItem, OptionChainRequest, OptionContractItem, OptionKlineItem, OptionKlineRequest, OptionQuoteRequest,
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
async fn test_get_market_state_parses_typed() {
    let server =
        mock_success_server(r#"[{"market":"US","marketStatus":"Trading","status":"TRADING","openTime":"09:30"}]"#)
            .await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));

    let states = qc.get_market_state("US").await.unwrap();
    assert_eq!(states.len(), 1);
    assert_eq!(states[0].market, "US");
    assert_eq!(states[0].market_status, "Trading");
    assert_eq!(states[0].open_time, "09:30");
}

#[tokio::test]
async fn test_get_real_time_quote_parses_typed() {
    let server = mock_success_server(
        r#"[{"symbol":"AAPL","latestPrice":150.0,"askPrice":150.1,"askSize":100,"volume":1000000}]"#,
    )
    .await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));

    let briefs = qc
        .get_real_time_quote(BriefRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(briefs.len(), 1);
    assert_eq!(briefs[0].symbol, "AAPL");
    assert_eq!(briefs[0].latest_price, 150.0);
    assert_eq!(briefs[0].ask_price, 150.1);
}

#[tokio::test]
async fn test_get_kline_parses_typed() {
    let server = mock_success_server(
        r#"[{"symbol":"AAPL","period":"day","items":[{"time":1700000000,"open":150.0,"close":151.0,"high":152.0,"low":149.0,"volume":1000}]}]"#,
    )
    .await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));

    let kline = qc.get_kline(KlineRequest { symbols: Some(vec!["AAPL".into()]), period: Some("day".into()), ..Default::default() }).await.unwrap();
    assert_eq!(kline.len(), 1);
    assert_eq!(kline[0].symbol, "AAPL");
    assert_eq!(kline[0].items.len(), 1);
    assert_eq!(kline[0].items[0].open, 150.0);
}

#[tokio::test]
async fn test_get_quote_depth_parses_typed() {
    let server = mock_success_server(
        r#"[{"symbol":"AAPL","asks":[{"price":150.0,"count":1,"volume":100}],"bids":[{"price":149.5,"count":1,"volume":200}]}]"#,
    )
    .await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));

    let d = qc
        .get_quote_depth(QuoteDepthRequest {
            symbols: Some(vec!["AAPL".into()]),
            market: Some("US".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].symbol, "AAPL");
    assert_eq!(d[0].asks[0].price, 150.0);
    assert_eq!(d[0].bids[0].volume, 200);
}

#[tokio::test]
async fn test_grab_quote_permission_parses_typed() {
    let server = mock_success_server(r#"[{"name":"usStockQuote","expireAt":1700000000}]"#).await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));

    let ps = qc.grab_quote_permission().await.unwrap();
    assert_eq!(ps.len(), 1);
    assert_eq!(ps[0].name, "usStockQuote");
    assert_eq!(ps[0].expire_at, 1700000000);
}

#[tokio::test]
async fn test_get_corporate_action_flattens_grouped() {
    let server = mock_success_server(
        r#"{"AAPL":[{"symbol":"AAPL","market":"US","actionType":"DIVIDEND","executeDate":"2025-01-01","amount":0.25}]}"#,
    )
    .await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));

    let actions = qc
        .get_corporate_action(CorporateActionRequest {
            symbols: vec!["AAPL".into()],
            market: "US".into(),
            action_type: "DIVIDEND".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].symbol, "AAPL");
    assert_eq!(actions[0].action_type, "DIVIDEND");
    assert_eq!(actions[0].amount, 0.25);
}

#[tokio::test]
async fn test_get_capital_distribution_option_some() {
    let server = mock_success_server(
        r#"{"symbol":"AAPL","netInflow":1000.0,"inAll":2000.0,"inBig":500.0}"#,
    )
    .await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));

    let cd = qc.get_capital_distribution("AAPL", "US").await.unwrap();
    let cd = cd.expect("should have data");
    assert_eq!(cd.symbol, "AAPL");
    assert_eq!(cd.net_inflow, 1000.0);
}

#[tokio::test]
async fn test_market_scanner_typed() {
    let server = mock_success_server(
        r#"{"page":0,"totalPage":1,"totalCount":1,"pageSize":10,"items":[{"symbol":"AAPL","market":"US"}]}"#,
    )
    .await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));

    let res = qc
        .market_scanner(MarketScannerRequest {
            market: "US".into(),
            page: Some(0),
            page_size: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();
    let res = res.expect("scanner should return data");
    assert_eq!(res.total_count, 1);
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].symbol, "AAPL");
}

// ========== 请求参数（snake_case wire format） 测试 ==========

#[tokio::test]
async fn test_get_market_state_sends_snake_case_and_method() {
    let server = mock_success_server(r#"[]"#).await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_market_state("US").await;

    let received = server.received_requests().await.unwrap();
    let req_body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req_body["method"].as_str().unwrap(), "market_state");
    let biz = biz_of(&received[0]);
    assert_eq!(biz["market"].as_str().unwrap(), "US");
}

#[tokio::test]
async fn test_get_real_time_quote_uses_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_real_time_quote(BriefRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await;

    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "quote_real_time");
}

#[tokio::test]
async fn test_get_future_contracts_sends_exchange_code() {
    let server = mock_success_server(r#"[]"#).await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_future_contracts("CME").await;

    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "future_contract_by_exchange_code");
    let biz = biz_of(&received[0]);
    assert_eq!(biz["exchange_code"].as_str().unwrap(), "CME");
}

#[tokio::test]
async fn test_get_future_real_time_quote_sends_contract_codes() {
    let server = mock_success_server(r#"[]"#).await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_future_real_time_quote(FutureRealTimeQuoteRequest {
            contract_codes: Some(vec!["CL2609".into()]),
            ..Default::default()
        })
        .await;

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert_eq!(biz["contract_codes"][0].as_str().unwrap(), "CL2609");
}

#[tokio::test]
async fn test_get_future_kline_snake_case_wire() {
    let server = mock_success_server(r#"[]"#).await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_future_kline(FutureKlineRequest {
            contract_codes: Some(vec!["CL2609".into()]),
            period: Some("day".into()),
            begin_time: Some(-1),
            end_time: Some(-1),
            ..Default::default()
        })
        .await;

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert!(biz.get("contract_codes").is_some());
    assert!(biz.get("begin_time").is_some());
    assert!(biz.get("end_time").is_some());
    assert!(biz.get("contractCodes").is_none());
    assert!(biz.get("beginTime").is_none());
}

#[tokio::test]
async fn test_get_financial_daily_wire_snake_case() {
    let server = mock_success_server(r#"[]"#).await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_financial_daily(FinancialDailyRequest {
            symbols: vec!["AAPL".into()],
            market: "US".into(),
            fields: vec!["shares_outstanding".into()],
            begin_date: "2025-01-01".into(),
            end_date: "2025-01-31".into(),
        })
        .await;

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert!(biz.get("begin_date").is_some());
    assert!(biz.get("end_date").is_some());
    assert!(biz.get("beginDate").is_none());
}

#[tokio::test]
async fn test_get_financial_report_wire_snake_case() {
    let server = mock_success_server(r#"[]"#).await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_financial_report(FinancialReportRequest {
            symbols: vec!["AAPL".into()],
            market: "US".into(),
            fields: vec!["total_revenue".into()],
            period_type: "Annual".into(),
            ..Default::default()
        })
        .await;

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert_eq!(biz["period_type"].as_str().unwrap(), "Annual");
    assert!(biz.get("periodType").is_none());
}

#[tokio::test]
async fn test_get_option_chain_sends_expiry_ms() {
    let server = mock_success_server(r#"[]"#).await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_option_chain(OptionChainRequest::new(vec![
        OptionChainItem::from_date("AAPL", "2024-01-19").unwrap(),
    ])).await;

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    let basic = &biz["option_basic"][0];
    assert_eq!(basic["symbol"].as_str().unwrap(), "AAPL");
    // 2024-01-19 00:00:00 America/New_York = 1705640400000
    assert_eq!(basic["expiry"].as_i64().unwrap(), 1705640400000);
}

#[tokio::test]
async fn test_get_option_quote_parses_identifier() {
    let server = mock_success_server(r#"[]"#).await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_option_quote(OptionQuoteRequest::new(vec![
        OptionContractItem::from_occ("AAPL 240119C00150000").unwrap(),
    ])).await;

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    let basic = &biz["option_basic"][0];
    assert_eq!(basic["symbol"].as_str().unwrap(), "AAPL");
    assert_eq!(basic["right"].as_str().unwrap(), "CALL");
    assert_eq!(basic["strike"].as_f64().unwrap(), 150.0);
    // 2024-01-19 00:00:00 America/New_York = 1705640400000
    assert_eq!(basic["expiry"].as_i64().unwrap(), 1705640400000);
}

#[tokio::test]
async fn test_get_option_kline_uses_option_query_key() {
    let server = mock_success_server(r#"[]"#).await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_option_kline(OptionKlineRequest {
        option_query: Some(vec![
            OptionKlineItem::from_occ("AAPL 240119C00150000", "day").unwrap(),
        ]),
        ..Default::default()
    }).await;

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert!(biz.get("option_query").is_some());
    assert_eq!(biz["option_query"][0]["period"].as_str().unwrap(), "day");
}

#[tokio::test]
async fn test_get_future_exchange_sends_sec_type_fut() {
    let server = mock_success_server(r#"[]"#).await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_future_exchange().await;

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert_eq!(biz["sec_type"].as_str().unwrap(), "FUT");
}

#[tokio::test]
async fn test_quote_api_error() {
    let server = mock_error_server(2100, "行情查询失败").await;
        let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    assert!(qc.get_market_state("US").await.is_err());
}

// ========== call_* 低级接口直接调用测试 ==========

#[tokio::test]
async fn test_call_into_parses_typed() {
    let server = mock_success_server(
        r#"[{"market":"US","marketStatus":"Trading","status":"TRADING","openTime":"09:30"}]"#,
    )
    .await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let states: Vec<MarketState> = qc
        .call_into("market_state", serde_json::json!({"market": "US"}))
        .await
        .unwrap();
    assert_eq!(states.len(), 1);
    assert_eq!(states[0].market, "US");
}

#[tokio::test]
async fn test_call_into_versioned_uses_version() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let states: Vec<MarketState> = qc
        .call_into_versioned("market_state", serde_json::json!({}), Some("1.0"))
        .await
        .unwrap();
    assert!(states.is_empty());
    let reqs = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&reqs[0].body).unwrap();
    assert_eq!(body["version"].as_str().unwrap(), "1.0");
}

#[tokio::test]
async fn test_call_into_items_unwraps_items_key() {
    let server = mock_success_server(
        r#"{"items":[{"symbol":"AAPL","latestPrice":150.0}]}"#,
    )
    .await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let briefs: Vec<Brief> = qc
        .call_into_items("brief", serde_json::json!({"symbols": ["AAPL"]}))
        .await
        .unwrap();
    assert_eq!(briefs.len(), 1);
    assert_eq!(briefs[0].symbol, "AAPL");
}

#[tokio::test]
async fn test_call_into_items_empty_on_null_data() {
    let server = mock_success_server("null").await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let briefs: Vec<Brief> = qc
        .call_into_items("brief", serde_json::json!({}))
        .await
        .unwrap();
    assert!(briefs.is_empty());
}

#[tokio::test]
async fn test_call_into_list_or_object_handles_array() {
    let server = mock_success_server(
        r#"[{"market":"US","marketStatus":"Trading","status":"TRADING","openTime":"09:30"}]"#,
    )
    .await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let states: Vec<MarketState> = qc
        .call_into_list_or_object("market_state", serde_json::json!({}))
        .await
        .unwrap();
    assert_eq!(states.len(), 1);
}

#[tokio::test]
async fn test_call_into_list_or_object_handles_single_object() {
    let server = mock_success_server(
        r#"{"market":"US","marketStatus":"Trading","status":"TRADING","openTime":"09:30"}"#,
    )
    .await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let states: Vec<MarketState> = qc
        .call_into_list_or_object("market_state", serde_json::json!({}))
        .await
        .unwrap();
    assert_eq!(states.len(), 1);
    assert_eq!(states[0].market, "US");
}

#[tokio::test]
async fn test_call_optional_returns_some() {
    let server = mock_success_server(
        r#"{"symbol":"AAPL","latestPrice":150.0}"#,
    )
    .await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let result: Option<Brief> = qc
        .call_optional("brief", serde_json::json!({"symbols": ["AAPL"]}))
        .await
        .unwrap();
    let b = result.expect("should be Some");
    assert_eq!(b.symbol, "AAPL");
}

#[tokio::test]
async fn test_call_optional_returns_none_on_null_data() {
    let server = mock_success_server("null").await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let result: Option<Brief> = qc
        .call_optional("brief", serde_json::json!({}))
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_call_optional_versioned_passes_version() {
    let server = mock_success_server("null").await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _: Option<Brief> = qc
        .call_optional_versioned("brief", serde_json::json!({}), Some("3.0"))
        .await
        .unwrap();
    let reqs = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&reqs[0].body).unwrap();
    assert_eq!(body["version"].as_str().unwrap(), "3.0");
}
