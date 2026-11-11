//! News example - covers the two NewsClient public methods.
//!
//! Config is auto-discovered from:
//!   1. ./tiger_openapi_config.properties
//!   2. ~/.tigeropen/tiger_openapi_config.properties
//!
//! TIGER_CONFIG_PATH env var overrides both.
//!
//! News requests are routed to the main gateway (config.server_url), not the quote server.
//!
//! Run: `TIGER_CONFIG_PATH=~/.tigeropen/tiger_openapi_config.properties cargo run --example news_example`

use tigeropen::config::ClientConfig;
use tigeropen::model::enums::Language;
use tigeropen::model::news_requests::{NewsHighlightListRequest, NewsStockListRequest};
use tigeropen::news::NewsClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = match std::env::var("TIGER_CONFIG_PATH") {
        Ok(path) => ClientConfig::builder().properties_file(&path).build()?,
        Err(_) => ClientConfig::builder().build()?,
    };
    println!("tiger_id={} account={}\n", config.tiger_id, config.account);

    let client = NewsClient::from_config(config);

    // 1. 个股新闻列表（news_stock_list）：单标的，page_count / page_size 翻页
    match client
        .get_stock_news(NewsStockListRequest {
            symbol: "AAPL".to_string(),
            lang: Some(Language::ZhCn),
            page_count: Some(1),
            page_size: Some(20),
        })
        .await
    {
        Ok(page) if !page.items.is_empty() => {
            println!(
                "[ OK ] GetStockNews(AAPL) pageCount={} pageSize={} totalSize={}",
                page.page_count, page.page_size, page.total_size
            );
            for item in page.items.iter().take(3) {
                println!(
                    "       {} — {} (pubTimestamp={})",
                    item.title, item.url, item.pub_timestamp
                );
            }
            if page.items.len() > 3 {
                println!("       ... and {} more", page.items.len() - 3);
            }
        }
        Ok(page) => println!(
            "[ OK ] GetStockNews(AAPL) (empty: {} items)",
            page.items.len()
        ),
        Err(e) => println!("[FAIL] GetStockNews(AAPL): {}", e),
    }

    // 2. 市场要闻列表（news_highlight_list）：last_id 游标翻页（该接口无 page_count / page_size）
    let mut last_id: Option<String> = None;
    let mut page_no = 0;
    loop {
        page_no += 1;
        let page = match client
            .get_market_news(NewsHighlightListRequest {
                last_id: last_id.clone(),
                lang: Some(Language::ZhCn),
            })
            .await
        {
            Ok(page) => page,
            Err(e) => {
                println!("[FAIL] GetMarketNews(page {}): {}", page_no, e);
                break;
            }
        };
        if page.items.is_empty() {
            println!("[ OK ] GetMarketNews(page {}) (end of feed)", page_no);
            break;
        }
        println!(
            "[ OK ] GetMarketNews(page {}) items={} totalSize={}",
            page_no,
            page.items.len(),
            page.total_size
        );
        for item in page.items.iter().take(3) {
            println!("       {} — {}", item.title, item.url);
        }
        // 下一页：把本页最后一条的 id 作为 last_id 传回
        last_id = page.items.last().map(|item| item.id.clone());
        if page_no >= 3 {
            println!("       (stopping after 3 pages for the example)");
            break;
        }
    }

    Ok(())
}
