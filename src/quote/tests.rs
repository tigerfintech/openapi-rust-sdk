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
    CorporateActionRequest, FinancialDailyRequest, FinancialReportRequest, MarketScannerRequest,
};
use crate::model::quote_requests::{
    BriefRequest, FutureKlineRequest, FutureRealTimeQuoteRequest, KlineRequest,
    OptionAnalysisSymbol, OptionChainFilter, OptionChainItem, OptionChainRequest,
    OptionContractItem, OptionKlineItem, OptionKlineRequest, OptionQuoteRequest, QuoteDepthRequest,
    RangeF64,
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
    let server = mock_success_server(
        r#"[{"market":"US","marketStatus":"Trading","status":"TRADING","openTime":"09:30"}]"#,
    )
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
async fn test_get_quote_overnight_parses_all_fields() {
    let server = mock_success_server(
        r#"[{"symbol":"AAPL","latestPrice":234.56,"askPrice":234.6,"askSize":120,"bidPrice":234.5,"bidSize":80,"preClose":230.0,"volume":12345,"amount":2895643.21,"timestamp":1723456789000,"tradingStatus":1,"change":4.56,"changeRate":0.019826,"amplitude":0.025}]"#,
    )
    .await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));

    let quotes = qc
        .get_quote_overnight(QuoteOvernightRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(quotes.len(), 1);
    let quote = &quotes[0];
    assert_eq!(quote.symbol, "AAPL");
    assert_eq!(quote.latest_price, 234.56);
    assert_eq!(quote.ask_price, 234.6);
    assert_eq!(quote.ask_size, 120);
    assert_eq!(quote.bid_price, 234.5);
    assert_eq!(quote.bid_size, 80);
    assert_eq!(quote.pre_close, 230.0);
    assert_eq!(quote.volume, 12_345);
    assert_eq!(quote.amount, 2_895_643.21);
    assert_eq!(quote.timestamp, 1_723_456_789_000);
    assert_eq!(quote.trading_status, 1);
    assert_eq!(quote.change, 4.56);
    assert_eq!(quote.change_rate, 0.019826);
    assert_eq!(quote.amplitude, 0.025);
}

#[tokio::test]
async fn test_get_kline_parses_typed() {
    let server = mock_success_server(
        r#"[{"symbol":"AAPL","period":"day","items":[{"time":1700000000,"open":150.0,"close":151.0,"high":152.0,"low":149.0,"volume":1000}]}]"#,
    )
    .await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));

    let kline = qc
        .get_kline(KlineRequest {
            symbols: Some(vec!["AAPL".into()]),
            period: Some("day".into()),
            ..Default::default()
        })
        .await
        .unwrap();
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
async fn test_get_corporate_symbol_change() {
    let server = mock_success_server(
        r#"{"META":[{"symbol":"META","market":"US","actionType":"symbol_change","executeDate":"2022-06-09","oldSymbol":"FB","newSymbol":"META"}]}"#,
    )
    .await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let items = qc
        .get_corporate_symbol_change(CorporateActionRequest {
            symbols: vec!["META".into()],
            market: "US".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].old_symbol, "FB");
    assert_eq!(items[0].new_symbol, "META");
}

#[tokio::test]
async fn test_get_corporate_delisting() {
    let server = mock_success_server(
        r#"{"TWTR":[{"symbol":"TWTR","market":"US","actionType":"delisting","executeDate":"2022-10-28","announcedDate":"2022-10-27","reason":"acquired"}]}"#,
    )
    .await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let items = qc
        .get_corporate_delisting(CorporateActionRequest {
            symbols: vec!["TWTR".into()],
            market: "US".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].reason, "acquired");
}

#[tokio::test]
async fn test_get_corporate_ipo() {
    let server = mock_success_server(
        r#"{"RIVN":[{"symbol":"RIVN","market":"US","actionType":"ipo","executeDate":"2021-11-10","ipoName":"Rivian Automotive","listingDate":"2021-11-10","listingPrice":78.0,"sharesOutstanding":864000000,"sharesFloat":153000000,"offerAmount":11932000000.0,"priceRange":"72-74","currency":"USD","minPurchaseQuantity":1,"leverageRatio":1.0}]}"#,
    )
    .await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let items = qc
        .get_corporate_ipo(CorporateActionRequest {
            symbols: vec!["RIVN".into()],
            market: "US".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].ipo_name, "Rivian Automotive");
    assert!((items[0].listing_price - 78.0).abs() < 0.001);
}

#[tokio::test]
async fn test_get_capital_distribution_option_some() {
    let server =
        mock_success_server(r#"{"symbol":"AAPL","netInflow":1000.0,"inAll":2000.0,"inBig":500.0}"#)
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
    assert_eq!(
        req["method"].as_str().unwrap(),
        "future_contract_by_exchange_code"
    );
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
    let _ = qc
        .get_option_chain(OptionChainRequest::new(vec![OptionChainItem::from_date(
            "AAPL",
            "2024-01-19",
        )
        .unwrap()]))
        .await;

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
    let _ = qc
        .get_option_quote(OptionQuoteRequest::new(vec![OptionContractItem::from_occ(
            "AAPL 240119C00150000",
        )
        .unwrap()]))
        .await;

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    let basic = &biz["option_basic"][0];
    assert_eq!(basic["symbol"].as_str().unwrap(), "AAPL");
    assert_eq!(basic["right"].as_str().unwrap(), "CALL");
    assert_eq!(basic["strike"].as_str().unwrap(), "150.0");
    // 2024-01-19 00:00:00 America/New_York = 1705640400000
    assert_eq!(basic["expiry"].as_i64().unwrap(), 1705640400000);
}

#[tokio::test]
async fn test_get_option_kline_uses_option_query_key() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_option_kline(OptionKlineRequest {
            option_query: Some(vec![OptionKlineItem::from_occ(
                "AAPL 240119C00150000",
                "day",
            )
            .unwrap()]),
            ..Default::default()
        })
        .await;

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert!(biz.get("option_query").is_some());
    assert_eq!(biz["option_query"][0]["period"].as_str().unwrap(), "day");
}

#[tokio::test]
async fn test_option_chain_return_greek_value_serialized() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_option_chain(OptionChainRequest {
            option_basic: Some(vec![
                OptionChainItem::from_date("AAPL", "2024-01-19").unwrap()
            ]),
            return_greek_value: Some(true),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert_eq!(biz["return_greek_value"].as_bool().unwrap(), true);
}

#[tokio::test]
async fn test_option_chain_filter_serialized() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_option_chain(OptionChainRequest {
            option_basic: Some(vec![
                OptionChainItem::from_date("AAPL", "2024-01-19").unwrap()
            ]),
            option_filter: Some(OptionChainFilter {
                in_the_money: Some(true),
                implied_volatility: Some(RangeF64::new(0.1, 1.0)),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert_eq!(
        biz["option_filter"]["in_the_money"].as_bool().unwrap(),
        true
    );
    assert!(
        (biz["option_filter"]["implied_volatility"]["min"]
            .as_f64()
            .unwrap()
            - 0.1)
            .abs()
            < 1e-9
    );
    assert!(
        (biz["option_filter"]["implied_volatility"]["max"]
            .as_f64()
            .unwrap()
            - 1.0)
            .abs()
            < 1e-9
    );
}

#[tokio::test]
async fn test_option_kline_sort_dir_serialized() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let mut item = OptionKlineItem::from_occ("AAPL 240119C00150000", "day").unwrap();
    item.sort_dir = Some("asc".to_string());
    let _ = qc
        .get_option_kline(OptionKlineRequest {
            option_query: Some(vec![item]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert_eq!(biz["option_query"][0]["sort_dir"].as_str().unwrap(), "asc");
}

#[test]
fn test_option_analysis_symbol_require_volatility_list() {
    let sym = OptionAnalysisSymbol {
        symbol: "AAPL".to_string(),
        period: Some("day".to_string()),
        require_volatility_list: Some(true),
    };
    let json = serde_json::to_value(&sym).unwrap();
    assert_eq!(json["symbol"].as_str().unwrap(), "AAPL");
    assert_eq!(json["require_volatility_list"].as_bool().unwrap(), true);
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
    let server = mock_success_server(r#"{"items":[{"symbol":"AAPL","latestPrice":150.0}]}"#).await;
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
    let server = mock_success_server(r#"{"symbol":"AAPL","latestPrice":150.0}"#).await;
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

// ========== 缺失方法的 wire method 测试 ==========

use crate::model::quote_requests::{
    AllFutureContractsRequest, DelayedQuoteRequest, FinancialCurrencyRequest,
    FinancialExchangeRateRequest, FundContractsRequest, FundHistoryQuoteRequest, FundQuoteRequest,
    FundSymbolsRequest, FutureContinuousContractsRequest, FutureContractSingleRequest,
    FutureDepthRequest, FutureHistoryMainContractRequest, FutureKlineByPageRequest,
    FutureTradeTicksRequest, FutureTradingTimesRequest, IndustryListRequest, IndustryStocksRequest,
    KlineByPageRequest, KlineQuotaRequest, MarketScannerTagsRequest, OptionAnalysisRequest,
    OptionDepthRequest, OptionSymbolsRequest, OptionTimelineRequest, OptionTradeTicksRequest,
    QuoteOvernightRequest, QuotePermissionRequest, ShortInterestRequest, StockBrokerRequest,
    StockDetailsRequest, StockFundamentalRequest, StockIndustryRequest, SymbolsRequest,
    TimelineHistoryRequest, TradeMetasRequest, TradeRankRequest, TradeTickRequest,
    TradingCalendarRequest, WarrantFilterRequest, WarrantQuoteRequest,
};

// --- 1. get_brief (deprecated, calls get_real_time_quote) ---

#[tokio::test]
#[allow(deprecated)]
async fn test_get_brief_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_brief(BriefRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "quote_real_time");
}

// --- 2. get_timeline ---

#[tokio::test]
async fn test_get_timeline_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_timeline(&["AAPL"]).await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "timeline");
    assert_eq!(req["version"].as_str().unwrap(), "3.0");
}

// --- 3. get_trade_tick ---

#[tokio::test]
async fn test_get_trade_tick_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_trade_tick(TradeTickRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "trade_tick");
}

#[tokio::test]
async fn test_get_trade_tick_decodes_part_code_and_part_name() {
    // REST response carries partCode/partName as server-sent strings;
    // verify TradeTickItem deserializes them correctly.
    let server = mock_success_server(
        r#"[{
        "symbol": "AAPL",
        "beginIndex": 0,
        "endIndex": 1,
        "items": [
            {"time": 1700000000000, "volume": 100, "price": 150.25, "type": "+",
             "partCode": "NYSE", "partName": "New York Stock Exchange, LLC (NYSE)"},
            {"time": 1700000001000, "volume": 200, "price": 150.50, "type": "-",
             "partCode": "NSDQ", "partName": "NASDAQ Stock Market, LLC (NASDAQ)"}
        ]
    }]"#,
    )
    .await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let result = qc
        .get_trade_tick(TradeTickRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    let items = &result[0].items;
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].part_code, "NYSE");
    assert_eq!(items[0].part_name, "New York Stock Exchange, LLC (NYSE)");
    assert_eq!(items[1].part_code, "NSDQ");
    assert_eq!(items[1].part_name, "NASDAQ Stock Market, LLC (NASDAQ)");
}

// --- 4. get_symbols ---

#[tokio::test]
async fn test_get_symbols_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_symbols(SymbolsRequest {
            market: Some("US".into()),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "all_symbols");
}

#[tokio::test]
async fn test_get_symbols_returns_vec_string() {
    let server = mock_success_server(r#"["AAPL","GOOG","TSLA"]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let result = qc.get_symbols(SymbolsRequest::default()).await.unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], "AAPL");
}

// --- 5. get_symbol_names ---

#[tokio::test]
async fn test_get_symbol_names_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_symbol_names(SymbolsRequest::default()).await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "all_symbol_names");
}

// --- 6. get_trade_metas ---

#[tokio::test]
async fn test_get_trade_metas_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_trade_metas(TradeMetasRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "quote_stock_trade");
}

// --- 7. get_stock_details ---

#[tokio::test]
async fn test_get_stock_details_wire_method() {
    let server = mock_success_server(r#"{"items":[]}"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_stock_details(StockDetailsRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "stock_detail");
}

// --- 8. get_delayed_quote ---

#[tokio::test]
async fn test_get_delayed_quote_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_delayed_quote(DelayedQuoteRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "quote_delay");
}

// --- 9. get_stock_delay_briefs (deprecated) ---

#[tokio::test]
#[allow(deprecated)]
async fn test_get_stock_delay_briefs_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_stock_delay_briefs(DelayedQuoteRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "quote_delay");
}

// --- 10. get_kline_by_page ---

#[tokio::test]
async fn test_get_kline_by_page_wire_method() {
    let server = mock_success_server(
        r#"[{"symbol":"AAPL","period":"day","items":[{"time":1700000000,"open":150.0,"close":151.0,"high":152.0,"low":149.0,"volume":1000}]}]"#,
    )
    .await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_kline_by_page(KlineByPageRequest {
            symbols: Some(vec!["AAPL".into()]),
            period: Some("day".into()),
            page_size: Some(10),
            total_size: Some(10),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "kline");
}

// --- 11. get_timeline_history ---

#[tokio::test]
async fn test_get_timeline_history_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_timeline_history(TimelineHistoryRequest {
            symbols: Some(vec!["AAPL".into()]),
            date: Some("2025-01-01".into()),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "history_timeline");
}

// --- 12. get_trade_rank ---

#[tokio::test]
async fn test_get_trade_rank_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_trade_rank(TradeRankRequest {
            market: Some("US".into()),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "trade_rank");
}

// --- 13. get_short_interest ---

#[tokio::test]
async fn test_get_short_interest_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_short_interest(ShortInterestRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "quote_shortable_stocks");
}

// --- 14. get_stock_broker ---

#[tokio::test]
async fn test_get_stock_broker_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_stock_broker(StockBrokerRequest {
            symbol: Some("00700".into()),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "stock_broker");
}

// --- 15. get_stock_fundamental ---

#[tokio::test]
async fn test_get_stock_fundamental_wire_method() {
    let server = mock_success_server(r#"{}"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_stock_fundamental(StockFundamentalRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "stock_fundamental");
}

// --- 16. get_stock_industry ---

#[tokio::test]
async fn test_get_stock_industry_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_stock_industry(StockIndustryRequest {
            symbol: Some("AAPL".into()),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "stock_industry");
}

// --- 17. get_quote_permission ---

#[tokio::test]
async fn test_get_quote_permission_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_quote_permission(QuotePermissionRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "get_quote_permission");
}

// --- 18. get_kline_quota ---

#[tokio::test]
async fn test_get_kline_quota_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_kline_quota(KlineQuotaRequest::default()).await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "kline_quota");
}

// --- 19. get_option_expiration ---

#[tokio::test]
async fn test_get_option_expiration_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_option_expiration(&["AAPL"], None).await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "option_expiration");
}

#[tokio::test]
async fn test_get_option_expiration_with_market() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_option_expiration(&["00700"], Some("HK")).await;
    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert_eq!(biz["market"].as_str().unwrap(), "HK");
    assert_eq!(biz["symbols"][0].as_str().unwrap(), "00700");
}

// --- 20. get_option_quote (option_brief) ---

#[tokio::test]
async fn test_get_option_quote_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_option_quote(OptionQuoteRequest::new(vec![OptionContractItem::new(
            "AAPL",
            1705640400000,
            "CALL",
            "150.0",
        )]))
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "option_brief");
    assert_eq!(req["version"].as_str().unwrap(), "2.0");
}

// --- 21. get_option_trade_ticks ---

#[tokio::test]
async fn test_get_option_trade_ticks_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_option_trade_ticks(OptionTradeTicksRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "option_trade_tick");
}

/// Regression: `biz_content` for `option_trade_tick` must be a top-level
/// JSON array, not an object with a `contracts` field. Server rejects
/// object form with `biz param error(failed to parse parameters in
/// 'biz_content')`. See `OptionTradeTicksRequest` custom `Serialize`.
#[tokio::test]
async fn test_option_trade_ticks_biz_content_is_top_level_array() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let req = OptionTradeTicksRequest {
        contracts: Some(vec![crate::model::quote_requests::OptionQueryItem {
            symbol: Some("AAPL".to_string()),
            expiry: Some(1_755_230_400_000),
            strike: Some("200".to_string()),
            right: Some("CALL".to_string()),
            ..Default::default()
        }]),
        lang: None,
    };
    let _ = qc.get_option_trade_ticks(req).await;
    let received = server.received_requests().await.unwrap();
    let outer: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    // Gateway wraps biz_content as a stringified JSON payload; parse again.
    let biz_str = outer["biz_content"]
        .as_str()
        .expect("biz_content is string");
    let biz: serde_json::Value = serde_json::from_str(biz_str).unwrap();
    // Must be an array, not an object with `contracts` key.
    assert!(
        biz.is_array(),
        "biz_content must be a top-level array, got {biz}"
    );
    assert_eq!(biz.as_array().unwrap().len(), 1);
    assert_eq!(biz[0]["symbol"].as_str().unwrap(), "AAPL");
    assert_eq!(biz[0]["right"].as_str().unwrap(), "CALL");
}

// --- 22. get_option_timeline ---

#[tokio::test]
async fn test_get_option_timeline_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_option_timeline(OptionTimelineRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "option_timeline");
}

// --- 23. get_option_depth ---

#[tokio::test]
async fn test_get_option_depth_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_option_depth(OptionDepthRequest::default()).await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "option_depth");
}

// --- 24. get_option_symbols ---

#[tokio::test]
async fn test_get_option_symbols_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_option_symbols(OptionSymbolsRequest::default()).await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "all_hk_option_symbols");
}

// --- 25. get_option_analysis ---

#[tokio::test]
async fn test_get_option_analysis_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_option_analysis(OptionAnalysisRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "option_analysis");
}

// --- 26. get_future_contract (single) ---

#[tokio::test]
async fn test_get_future_contract_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_future_contract(FutureContractSingleRequest {
            contract_code: Some("CL2609".into()),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(
        req["method"].as_str().unwrap(),
        "future_contract_by_contract_code"
    );
}

// --- 27. get_all_future_contracts ---

#[tokio::test]
async fn test_get_all_future_contracts_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_all_future_contracts(AllFutureContractsRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "future_contracts");
}

// --- 28. get_current_future_contract ---

#[tokio::test]
async fn test_get_current_future_contract_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_current_future_contract(FutureContractSingleRequest {
            contract_code: Some("CL".into()),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "future_current_contract");
}

// --- 29. get_future_continuous_contracts ---

#[tokio::test]
async fn test_get_future_continuous_contracts_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_future_continuous_contracts(FutureContinuousContractsRequest::default())
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(
        req["method"].as_str().unwrap(),
        "future_continuous_contracts"
    );
}

// --- 30. get_future_history_main_contract ---

#[tokio::test]
async fn test_get_future_history_main_contract_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_future_history_main_contract(FutureHistoryMainContractRequest {
            contract_codes: Some(vec!["CL".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "future_main_contract");
}

// --- 31. get_future_kline_by_page ---

#[tokio::test]
async fn test_get_future_kline_by_page_wire_method() {
    let server = mock_success_server(
        r#"[{"contractCode":"CL2609","items":[{"time":1700000000,"open":70.0,"close":71.0,"high":72.0,"low":69.0,"volume":500}]}]"#,
    )
    .await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_future_kline_by_page(FutureKlineByPageRequest {
            contract_code: Some("CL2609".into()),
            period: Some("day".into()),
            page_size: Some(10),
            total_size: Some(10),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "future_kline");
    let biz = biz_of(&received[0]);
    assert!(biz.get("contract_code").is_some());
}

// --- 32. get_future_trade_ticks ---

#[tokio::test]
async fn test_get_future_trade_ticks_wire_method() {
    let server = mock_success_server(r#"{"contractCode":"CL2609","items":[]}"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_future_trade_ticks(FutureTradeTicksRequest {
            contract_code: Some("CL2609".into()),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "future_tick");
    assert_eq!(req["version"].as_str().unwrap(), "3.0");
}

// --- 33. get_future_depth ---

#[tokio::test]
async fn test_get_future_depth_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_future_depth(FutureDepthRequest {
            contract_codes: Some(vec!["CL2609".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "future_depth");
}

// --- 34. get_future_trading_times ---

#[tokio::test]
async fn test_get_future_trading_times_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_future_trading_times(FutureTradingTimesRequest {
            contract_code: Some("CL2609".into()),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "future_trading_date");
}

// --- 35. get_fund_symbols ---

#[tokio::test]
async fn test_get_fund_symbols_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_fund_symbols(FundSymbolsRequest::default()).await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "fund_all_symbols");
}

// --- 36. get_fund_contracts ---

#[tokio::test]
async fn test_get_fund_contracts_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_fund_contracts(FundContractsRequest {
            symbols: Some(vec!["SPY".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "fund_contracts");
}

// --- 37. get_fund_quote ---

#[tokio::test]
async fn test_get_fund_quote_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_fund_quote(FundQuoteRequest {
            symbols: Some(vec!["SPY".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "fund_quote");
}

// --- 38. get_fund_history_quote ---

#[tokio::test]
async fn test_get_fund_history_quote_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_fund_history_quote(FundHistoryQuoteRequest {
            symbols: Some(vec!["SPY".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "fund_history_quote");
}

// --- 39. get_warrant_quote ---

#[tokio::test]
async fn test_get_warrant_quote_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_warrant_quote(WarrantQuoteRequest {
            symbols: Some(vec!["12345".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "warrant_briefs");
}

// --- 40. get_warrant_briefs (deprecated) ---

#[tokio::test]
#[allow(deprecated)]
async fn test_get_warrant_briefs_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_warrant_briefs(WarrantQuoteRequest {
            symbols: Some(vec!["12345".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "warrant_briefs");
}

// --- 41. get_warrant_filter ---

#[tokio::test]
async fn test_get_warrant_filter_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_warrant_filter(WarrantFilterRequest {
            symbol: Some("00700".into()),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "warrant_filter");
}

// --- 42. get_industry_list ---

#[tokio::test]
async fn test_get_industry_list_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_industry_list(IndustryListRequest::default()).await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "industry_list");
}

// --- 43. get_industry_stocks ---

#[tokio::test]
async fn test_get_industry_stocks_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_industry_stocks(IndustryStocksRequest {
            industry_id: Some("IND001".into()),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "industry_stock_list");
}

// --- 44. get_corporate_split ---

#[tokio::test]
async fn test_get_corporate_split_wire_method() {
    let server = mock_success_server(r#"{"AAPL":[]}"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_corporate_split(CorporateActionRequest {
            symbols: vec!["AAPL".into()],
            market: "US".into(),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "corporate_action");
    let biz = biz_of(&received[0]);
    assert_eq!(biz["action_type"].as_str().unwrap(), "split");
}

// --- 45. get_corporate_dividend ---

#[tokio::test]
async fn test_get_corporate_dividend_wire_method() {
    let server = mock_success_server(r#"{"AAPL":[]}"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_corporate_dividend(CorporateActionRequest {
            symbols: vec!["AAPL".into()],
            market: "US".into(),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert_eq!(biz["action_type"].as_str().unwrap(), "dividend");
}

// --- 46. get_corporate_earnings_calendar ---

#[tokio::test]
async fn test_get_corporate_earnings_calendar_wire_method() {
    let server = mock_success_server(r#"{"AAPL":[]}"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_corporate_earnings_calendar(CorporateActionRequest {
            symbols: vec!["AAPL".into()],
            market: "US".into(),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert_eq!(biz["action_type"].as_str().unwrap(), "earning");
}

// --- 47. get_financial_currency ---

#[tokio::test]
async fn test_get_financial_currency_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_financial_currency(FinancialCurrencyRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "financial_currency");
}

// --- 48. get_financial_exchange_rate ---

#[tokio::test]
async fn test_get_financial_exchange_rate_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_financial_exchange_rate(FinancialExchangeRateRequest {
            currency_list: Some(vec!["USD".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "financial_exchange_rate");
}

// --- 49. get_trading_calendar ---

#[tokio::test]
async fn test_get_trading_calendar_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_trading_calendar(TradingCalendarRequest {
            market: Some("US".into()),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "trading_calendar");
}

// --- 50. get_capital_flow ---

#[tokio::test]
async fn test_get_capital_flow_wire_method() {
    let server = mock_success_server(r#"null"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc.get_capital_flow("AAPL", "US", "day").await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "capital_flow");
    let biz = biz_of(&received[0]);
    assert_eq!(biz["symbol"].as_str().unwrap(), "AAPL");
    assert_eq!(biz["market"].as_str().unwrap(), "US");
    assert_eq!(biz["period"].as_str().unwrap(), "day");
}

// --- 51. get_market_scanner_tags ---

#[tokio::test]
async fn test_get_market_scanner_tags_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_market_scanner_tags(MarketScannerTagsRequest {
            market: Some("US".into()),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "market_scanner_tags");
}

// --- 52. get_quote_overnight ---

#[tokio::test]
async fn test_get_quote_overnight_wire_method() {
    let server = mock_success_server(r#"[]"#).await;
    let qc = QuoteClient::new(HttpClient::new(test_config(&server.uri())));
    let _ = qc
        .get_quote_overnight(QuoteOvernightRequest {
            symbols: Some(vec!["AAPL".into()]),
            ..Default::default()
        })
        .await;
    let received = server.received_requests().await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(req["method"].as_str().unwrap(), "quote_overnight");
}
