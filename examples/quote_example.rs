//! Quote example
//!
//! Demonstrates how to use QuoteClient to query market state, real-time quotes,
//! K-line data, depth, trade ticks, timelines, option expirations, futures
//! exchanges, and quote permissions.
//!
//! Config is auto-discovered from:
//!   1. ./tiger_openapi_config.properties
//!   2. ~/.tigeropen/tiger_openapi_config.properties
//!
//! Run: `cargo run --example quote_example`

use tigeropen::config::ClientConfig;
use tigeropen::client::http_client::HttpClient;
use tigeropen::quote::QuoteClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClientConfig::builder().build()?; // auto-discovers config
    println!("tiger_id: {}, account: {}", config.tiger_id, config.account);

    let http = HttpClient::new(config);
    let qc = QuoteClient::new(&http);

    // Market state
    println!("=== Market State (US) ===");
    let states = qc.market_state("US").await?;
    println!("{:#?}", states);

    // Real-time quotes for multiple symbols
    println!("\n=== Real-Time Quotes ===");
    let quotes = qc.quote_real_time(&["AAPL", "TSLA", "GOOG"]).await?;
    println!("{:#?}", quotes);

    // Timeline (intraday minute bars)
    println!("\n=== Timeline (AAPL) ===");
    let timeline = qc.timeline(&["AAPL"]).await?;
    println!("{:#?}", timeline);

    // Quote depth (level 2 order book)
    println!("\n=== Quote Depth (AAPL) ===");
    let depth = qc.quote_depth("AAPL").await?;
    println!("{:#?}", depth);

    // Trade ticks (recent trades)
    println!("\n=== Trade Ticks (AAPL) ===");
    let ticks = qc.trade_tick(&["AAPL"]).await?;
    println!("{:#?}", ticks);

    // K-line data (daily bars)
    println!("\n=== K-Line (AAPL, day) ===");
    let kline = qc.kline("AAPL", "day").await?;
    println!("{:#?}", kline);

    // Option expiration dates
    println!("\n=== Option Expiration (AAPL) ===");
    let expirations = qc.option_expiration("AAPL").await?;
    println!("{:#?}", expirations);

    // Futures exchange list
    println!("\n=== Future Exchanges ===");
    let exchanges = qc.future_exchange().await?;
    println!("{:#?}", exchanges);

    // Quote permissions
    println!("\n=== Quote Permissions ===");
    let permissions = qc.grab_quote_permission().await?;
    println!("{:#?}", permissions);

    Ok(())
}
