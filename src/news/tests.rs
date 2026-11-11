//! NewsClient 测试模块。
//!
//! 使用 wiremock mock HTTP 响应，验证两个资讯方法的请求构造（snake_case）、
//! 响应解析（camelCase）、主网关路由与游标翻页语义。不访问真实网络。

use super::*;
use std::sync::OnceLock;
use std::time::Duration;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use rsa::RsaPrivateKey;

use crate::client::http_client::HttpClient;
use crate::config::client_config::ClientConfig;
use crate::model::enums::Language;
use crate::model::news::NewsItem;
use crate::model::news_requests::{NewsHighlightListRequest, NewsStockListRequest};
use crate::quote::QuoteClient;

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

fn test_config(server_url: &str) -> ClientConfig {
    ClientConfig {
        tiger_id: "test_tiger_id".to_string(),
        private_key: cached_test_private_key().to_string(),
        account: "DU123456".to_string(),
        secret_key: None,
        license: None,
        language: Language::ZhCn,
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

fn biz_of(req: &wiremock::Request) -> serde_json::Value {
    let v: serde_json::Value = serde_json::from_slice(&req.body).unwrap();
    let s = v["biz_content"].as_str().unwrap();
    serde_json::from_str(s).unwrap()
}

fn method_of(req: &wiremock::Request) -> String {
    let v: serde_json::Value = serde_json::from_slice(&req.body).unwrap();
    v["method"].as_str().unwrap().to_string()
}

// ========== 个股新闻（news_stock_list） ==========

#[tokio::test]
async fn test_get_stock_news_request_wire_format() {
    let data = r#"{"items":[],"pageCount":1,"pageSize":20,"totalPage":0,"totalSize":0}"#;
    let server = mock_success_server(data).await;
    let client = NewsClient::from_config(test_config(&server.uri()));

    let page = client
        .get_stock_news(NewsStockListRequest {
            symbol: "AAPL".to_string(),
            page_count: Some(2),
            page_size: Some(50),
            lang: Some(Language::ZhCn),
        })
        .await;
    assert!(page.is_ok());

    let received = server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);
    assert_eq!(method_of(&received[0]), "news_stock_list");
    // biz_content 入参为 snake_case，可选字段显式传值时全部落 key
    let biz = biz_of(&received[0]);
    assert_eq!(biz["symbol"], "AAPL");
    assert_eq!(biz["page_count"], 2);
    assert_eq!(biz["page_size"], 50);
    assert_eq!(biz["lang"], "zh_CN");
}

#[tokio::test]
async fn test_get_stock_news_omits_absent_optional_params() {
    let data = r#"{"items":[],"pageCount":1,"pageSize":20,"totalPage":0,"totalSize":0}"#;
    let server = mock_success_server(data).await;
    let client = NewsClient::from_config(test_config(&server.uri()));

    let page = client
        .get_stock_news(NewsStockListRequest {
            symbol: "AAPL".to_string(),
            ..Default::default()
        })
        .await;
    assert!(page.is_ok());

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    // 缺省的可选参数不落 key（缺省语义由服务端归一化：page_count=1、page_size=20、lang=混合）
    assert_eq!(biz, serde_json::json!({"symbol": "AAPL"}));
}

#[tokio::test]
async fn test_get_stock_news_parses_camel_case_response() {
    let data = r#"{
        "items":[{
            "id":"123456",
            "title":"Apple Q2",
            "summary":"Earnings beat",
            "pubTimestamp":1729413600,
            "url":"https://example.com/news/123456"
        }],
        "pageCount":1,"pageSize":20,"totalPage":10,"totalSize":200
    }"#;
    let server = mock_success_server(data).await;
    let client = NewsClient::from_config(test_config(&server.uri()));

    let page = client
        .get_stock_news(NewsStockListRequest {
            symbol: "AAPL".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(page.page_count, 1);
    assert_eq!(page.page_size, 20);
    assert_eq!(page.total_page, 10);
    assert_eq!(page.total_size, 200);
    assert_eq!(page.items.len(), 1);
    assert_eq!(
        page.items[0],
        NewsItem {
            id: "123456".to_string(),
            title: "Apple Q2".to_string(),
            summary: "Earnings beat".to_string(),
            pub_timestamp: 1729413600,
            url: "https://example.com/news/123456".to_string(),
        }
    );
}

#[tokio::test]
async fn test_get_stock_news_empty_items_is_success() {
    // 下游返回空 items 属正常成功返回，不报错
    let data = r#"{"items":[],"pageCount":1,"pageSize":20,"totalPage":0,"totalSize":0}"#;
    let server = mock_success_server(data).await;
    let client = NewsClient::from_config(test_config(&server.uri()));

    let page = client
        .get_stock_news(NewsStockListRequest {
            symbol: "AAPL".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(page.items.is_empty());
    assert_eq!(page.total_size, 0);
}

#[tokio::test]
async fn test_get_stock_news_data_null_returns_default_page() {
    let server = mock_success_server("null").await;
    let client = NewsClient::from_config(test_config(&server.uri()));

    let page = client
        .get_stock_news(NewsStockListRequest {
            symbol: "AAPL".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(page, NewsPage::default());
}

#[tokio::test]
async fn test_get_stock_news_double_encoded_data() {
    // 服务端偶发把 data 双重编码为 JSON 字符串，须解一层再解析
    let mock_server = MockServer::start().await;
    let response_body = r#"{"code":0,"message":"success","data":"{\"items\":[{\"id\":\"789\",\"title\":\"Fed cuts rates\",\"summary\":\"FOMC\",\"pubTimestamp\":1761429600,\"url\":\"https://example.com/h?id=789\"}],\"pageCount\":1,\"pageSize\":20,\"totalPage\":5,\"totalSize\":100}","timestamp":1700000000}"#;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;
    let client = NewsClient::from_config(test_config(&mock_server.uri()));

    let page = client
        .get_stock_news(NewsStockListRequest {
            symbol: "AAPL".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].id, "789");
    assert_eq!(page.items[0].title, "Fed cuts rates");
    assert_eq!(page.items[0].pub_timestamp, 1761429600);
    assert_eq!(page.total_size, 100);
}

#[tokio::test]
async fn test_get_stock_news_api_error() {
    let mock_server = MockServer::start().await;
    let response_body = r#"{"code":1010,"message":"参数错误","data":null,"timestamp":1700000000}"#;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;
    let client = NewsClient::from_config(test_config(&mock_server.uri()));

    let result = client
        .get_stock_news(NewsStockListRequest {
            symbol: "AAPL".to_string(),
            ..Default::default()
        })
        .await;
    match result.unwrap_err() {
        TigerError::Api { code, message } => {
            assert_eq!(code, 1010);
            assert_eq!(message, "参数错误");
        }
        other => panic!("应返回 Api 错误，实际: {:?}", other),
    }
}

// ========== 市场要闻（news_highlight_list） ==========

#[tokio::test]
async fn test_get_market_news_request_wire_format() {
    let data = r#"{"items":[],"pageCount":1,"pageSize":20,"totalPage":0,"totalSize":0}"#;
    let server = mock_success_server(data).await;
    let client = NewsClient::from_config(test_config(&server.uri()));

    let page = client
        .get_market_news(NewsHighlightListRequest {
            last_id: Some("789".to_string()),
            lang: Some(Language::EnUs),
        })
        .await;
    assert!(page.is_ok());

    let received = server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);
    assert_eq!(method_of(&received[0]), "news_highlight_list");
    let biz = biz_of(&received[0]);
    assert_eq!(biz["last_id"], "789");
    assert_eq!(biz["lang"], "en_US");
}

#[tokio::test]
async fn test_get_market_news_omits_absent_params() {
    let data = r#"{"items":[],"pageCount":1,"pageSize":20,"totalPage":0,"totalSize":0}"#;
    let server = mock_success_server(data).await;
    let client = NewsClient::from_config(test_config(&server.uri()));

    // 全部缺省：不传 last_id 从最新开始，不传 lang 为中英混合内容
    let page = client
        .get_market_news(NewsHighlightListRequest::default())
        .await;
    assert!(page.is_ok());

    let received = server.received_requests().await.unwrap();
    let biz = biz_of(&received[0]);
    assert_eq!(biz, serde_json::json!({}));
}

#[tokio::test]
async fn test_get_market_news_parses_camel_case_response() {
    let data = r#"{
        "items":[{
            "id":"789",
            "title":"Fed cuts rates",
            "summary":"FOMC decision",
            "pubTimestamp":1761429600,
            "url":"https://example.com/highlight/detail?id=789"
        }],
        "pageCount":1,"pageSize":20,"totalPage":5,"totalSize":100
    }"#;
    let server = mock_success_server(data).await;
    let client = NewsClient::from_config(test_config(&server.uri()));

    let page = client
        .get_market_news(NewsHighlightListRequest {
            last_id: Some("789".to_string()),
            lang: Some(Language::EnUs),
        })
        .await
        .unwrap();
    // highlight 走游标翻页，服务端固定每页一游标，pageCount 恒为 1
    assert_eq!(page.page_count, 1);
    assert_eq!(page.total_page, 5);
    assert_eq!(page.total_size, 100);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].id, "789");
    assert_eq!(page.items[0].title, "Fed cuts rates");
    assert_eq!(page.items[0].pub_timestamp, 1761429600);
}

#[tokio::test]
async fn test_get_market_news_cursor_pagination() {
    // 游标翻页语义：取上一页 items 最后一条的 id 作为下一次调用的 last_id
    let page1 = r#"{
        "items":[
            {"id":"100","title":"first","summary":"s1","pubTimestamp":1761420000,"url":"u1"},
            {"id":"200","title":"last","summary":"s2","pubTimestamp":1761420600,"url":"u2"}
        ],
        "pageCount":1,"pageSize":20,"totalPage":5,"totalSize":100
    }"#;
    let server1 = mock_success_server(page1).await;
    let client = NewsClient::from_config(test_config(&server1.uri()));

    // 首页：不传 last_id，从最新开始
    let first = client
        .get_market_news(NewsHighlightListRequest::default())
        .await
        .unwrap();
    assert_eq!(first.items.len(), 2);
    let received1 = server1.received_requests().await.unwrap();
    assert!(biz_of(&received1[0]).get("last_id").is_none());

    // 下一页：把本页最后一条的 id 作为 last_id 传回
    let last_id = first.items.last().unwrap().id.clone();
    assert_eq!(last_id, "200");

    let page2 = r#"{
        "items":[{"id":"300","title":"next","summary":"s3","pubTimestamp":1761421200,"url":"u3"}],
        "pageCount":1,"pageSize":20,"totalPage":5,"totalSize":100
    }"#;
    let server2 = mock_success_server(page2).await;
    let client2 = NewsClient::from_config(test_config(&server2.uri()));
    let second = client2
        .get_market_news(NewsHighlightListRequest {
            last_id: Some(last_id),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(second.items.len(), 1);

    let received2 = server2.received_requests().await.unwrap();
    assert_eq!(biz_of(&received2[0])["last_id"], "200");
}

// ========== 语言参数（lang）wire 值域 ==========

#[test]
fn test_news_request_lang_wire_values() {
    // Language 枚举 serde rename 与服务端值域逐字一致
    for (lang, wire) in [
        (Language::ZhCn, "zh_CN"),
        (Language::ZhTw, "zh_TW"),
        (Language::EnUs, "en_US"),
    ] {
        let req = NewsStockListRequest {
            symbol: "AAPL".to_string(),
            lang: Some(lang.clone()),
            ..Default::default()
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["lang"], wire);

        let req = NewsHighlightListRequest {
            lang: Some(lang),
            ..Default::default()
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["lang"], wire);
    }

    // None = 不发送 lang 字段（服务端返回中英混合内容）
    let req = NewsStockListRequest {
        symbol: "AAPL".to_string(),
        ..Default::default()
    };
    let json = serde_json::to_value(&req).unwrap();
    assert!(json.get("lang").is_none());
    assert!(json.get("page_count").is_none());
    assert!(json.get("page_size").is_none());

    let req = NewsHighlightListRequest::default();
    let json = serde_json::to_value(&req).unwrap();
    assert!(json.get("lang").is_none());
    assert!(json.get("last_id").is_none());
}

// ========== 路由回归（主网关，非行情域名） ==========

/// news 系列请求必须路由到主网关（server_url），不得走行情域名（quote_server_url）；
/// 对照断言：QUOTE 类型请求仍走行情域名，防止路由逻辑被误改。
/// （对齐 Java SDK NewsApiLogicTest#testNewsRoutesToMainGatewayNotQuote）
#[tokio::test]
async fn test_news_routes_to_main_gateway_not_quote() {
    let main_server = MockServer::start().await;
    let quote_server = MockServer::start().await;

    // 主网关返回合法资讯数据；若 news 误发往行情域名，解析将失败且计数断言兜底
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"code":0,"message":"success","data":{"items":[{"id":"1","title":"t","summary":"s","pubTimestamp":1,"url":"u"}],"pageCount":1,"pageSize":20,"totalPage":1,"totalSize":1},"timestamp":1700000000}"#,
        ))
        .expect(2)
        .mount(&main_server)
        .await;
    // 行情域名挂行情形状的响应（对照组使用；若 news 误入此域名会解析失败）
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"code":0,"message":"success","data":[{"market":"US","marketStatus":"Trading","status":"TRADING","openTime":"09:30"}],"timestamp":1700000000}"#,
        ))
        .mount(&quote_server)
        .await;

    let mut config = test_config(&main_server.uri());
    config.quote_server_url = quote_server.uri();

    // news_stock_list / news_highlight_list 均走主网关
    let news_client = NewsClient::from_config(config.clone());
    let stock_page = news_client
        .get_stock_news(NewsStockListRequest {
            symbol: "AAPL".to_string(),
            ..Default::default()
        })
        .await;
    assert!(stock_page.is_ok(), "news_stock_list 应成功");
    let highlight_page = news_client
        .get_market_news(NewsHighlightListRequest::default())
        .await;
    assert!(highlight_page.is_ok(), "news_highlight_list 应成功");

    assert_eq!(
        main_server.received_requests().await.unwrap().len(),
        2,
        "两个 news 请求都应发往主网关"
    );
    assert_eq!(
        quote_server.received_requests().await.unwrap().len(),
        0,
        "news 请求不得发往行情域名"
    );
    let received = main_server.received_requests().await.unwrap();
    assert_eq!(method_of(&received[0]), "news_stock_list");
    assert_eq!(method_of(&received[1]), "news_highlight_list");

    // 对照组：QUOTE 类型请求走行情域名（QuoteClient::from_config 使用 with_quote_server）
    let quote_client = QuoteClient::from_config(config);
    let states = quote_client.get_market_state("US").await;
    assert!(states.is_ok(), "对照组行情请求应成功");
    assert_eq!(
        quote_server.received_requests().await.unwrap().len(),
        1,
        "QUOTE 类型应走行情域名（对照组）"
    );
    assert_eq!(
        main_server.received_requests().await.unwrap().len(),
        2,
        "对照组不得新增主网关请求"
    );
}

/// 用已有 HttpClient（经 HttpClient::new 构造，指向主网关）创建 NewsClient，同样走主网关。
#[tokio::test]
async fn test_news_client_new_with_main_gateway_http_client() {
    let data = r#"{"items":[],"pageCount":1,"pageSize":20,"totalPage":0,"totalSize":0}"#;
    let main_server = mock_success_server(data).await;
    let client = NewsClient::new(HttpClient::new(test_config(&main_server.uri())));

    let page = client
        .get_market_news(NewsHighlightListRequest::default())
        .await;
    assert!(page.is_ok());
    assert_eq!(main_server.received_requests().await.unwrap().len(), 1);
}
