//! Trade example
//!
//! Demonstrates how to use TradeClient to query orders, positions, assets,
//! active orders, and shows a sample order JSON for place_order.
//!
//! Config is auto-discovered from:
//!   1. ./tiger_openapi_config.properties
//!   2. ~/.tigeropen/tiger_openapi_config.properties
//!
//! Run: `cargo run --example trade_example`

use tigeropen::config::ClientConfig;
use tigeropen::client::http_client::HttpClient;
use tigeropen::trade::TradeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClientConfig::builder().build()?; // auto-discovers config
    println!("tiger_id: {}, account: {}", config.tiger_id, config.account);

    let account = config.account.clone();
    let http = HttpClient::new(config);
    let tc = TradeClient::new(&http, &account);

    // Query account assets
    println!("=== Assets ===");
    let assets = tc.assets().await?;
    println!("{:#?}", assets);

    // Query all orders
    println!("\n=== Orders ===");
    let orders = tc.orders().await?;
    println!("{:#?}", orders);

    // Query active (pending) orders
    println!("\n=== Active Orders ===");
    let active = tc.active_orders().await?;
    println!("{:#?}", active);

    // Query positions
    println!("\n=== Positions ===");
    let positions = tc.positions().await?;
    println!("{:#?}", positions);

    // Sample order JSON for place_order (not executed)
    // Uncomment the tc.place_order() call below to actually submit the order.
    println!("\n=== Sample Limit Order (not submitted) ===");
    let sample_order = serde_json::json!({
        "symbol": "AAPL",
        "secType": "STK",
        "action": "BUY",
        "orderType": "LMT",
        "totalQuantity": 1,
        "limitPrice": 150.0,
        "timeInForce": "DAY",
        "outsideRth": false,
    });
    println!("{}", serde_json::to_string_pretty(&sample_order)?);

    // To preview the order without submitting:
    // let preview = tc.preview_order(sample_order.clone()).await?;
    // println!("{:#?}", preview);

    // To actually place the order:
    // let result = tc.place_order(sample_order).await?;
    // println!("{:#?}", result);

    Ok(())
}
