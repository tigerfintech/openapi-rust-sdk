//! Push example
//!
//! Demonstrates how to connect to the push server, register callbacks for
//! all supported push data types, and subscribe to real-time market data
//! and account updates using PushClient.
//!
//! Config is auto-discovered from:
//!   1. ./tiger_openapi_config.properties
//!   2. ~/.tigeropen/tiger_openapi_config.properties
//!
//! Run: `cargo run --example push_example`

use std::sync::Arc;
use tigeropen::config::ClientConfig;
use tigeropen::push::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClientConfig::builder().build()?; // auto-discovers config
    println!("tiger_id: {}, account: {}", config.tiger_id, config.account);

    let account = config.account.clone();
    let pc = Arc::new(PushClient::new(config, None));

    // Register callbacks for all supported push data types
    pc.set_callbacks(Callbacks {
        // --- Market data callbacks ---
        on_quote: Some(Arc::new(|data| {
            println!(
                "[Quote] {} price={:?} volume={:?}",
                data.symbol, data.latest_price, data.volume
            );
        })),
        on_tick: Some(Arc::new(|data| {
            println!(
                "[Tick] {} sn={} prices={} volumes={}",
                data.symbol, data.sn, data.price.len(), data.volume.len()
            );
        })),
        on_depth: Some(Arc::new(|data| {
            let ask_levels = data.ask.as_ref().map_or(0, |a| a.price.len());
            let bid_levels = data.bid.as_ref().map_or(0, |b| b.price.len());
            println!(
                "[Depth] {} ask_levels={} bid_levels={}",
                data.symbol, ask_levels, bid_levels
            );
        })),
        on_kline: Some(Arc::new(|data| {
            println!(
                "[Kline] {} open={} close={} high={} low={} volume={}",
                data.symbol, data.open, data.close, data.high, data.low, data.volume
            );
        })),
        on_option: Some(Arc::new(|data| {
            println!(
                "[Option] {} price={:?} volume={:?}",
                data.symbol, data.latest_price, data.volume
            );
        })),
        on_future: Some(Arc::new(|data| {
            println!(
                "[Future] {} price={:?} volume={:?}",
                data.symbol, data.latest_price, data.volume
            );
        })),

        // --- Account data callbacks ---
        on_order: Some(Arc::new(|data| {
            println!(
                "[Order] id={} symbol={} status={} filled_qty={:?}",
                data.id, data.symbol, data.status, data.filled_quantity
            );
        })),
        on_position: Some(Arc::new(|data| {
            println!(
                "[Position] symbol={} qty={} avg_cost={} market_value={}",
                data.symbol, data.position, data.average_cost, data.market_value
            );
        })),
        on_transaction: Some(Arc::new(|data| {
            println!(
                "[Transaction] id={} symbol={} filled_price={} filled_qty={}",
                data.id, data.symbol, data.filled_price, data.filled_quantity
            );
        })),
        on_asset: Some(Arc::new(|data| {
            println!(
                "[Asset] account={} net_value={} buying_power={}",
                data.account, data.net_liquidation, data.buying_power
            );
        })),

        // --- Ranking / top data callbacks ---
        on_stock_top: Some(Arc::new(|data| {
            println!(
                "[StockTop] market={} entries={}",
                data.market,
                data.top_data.len()
            );
        })),
        on_option_top: Some(Arc::new(|data| {
            println!(
                "[OptionTop] market={} entries={}",
                data.market,
                data.top_data.len()
            );
        })),

        // --- Connection lifecycle callbacks ---
        on_connect: Some(Arc::new(|| println!("[Connected] push server connected"))),
        on_disconnect: Some(Arc::new(|| println!("[Disconnected] push server disconnected"))),
        on_error: Some(Arc::new(|msg| println!("[Error] {}", msg))),
        on_kickout: Some(Arc::new(|msg| println!("[Kickout] {}", msg))),

        // Remaining callbacks left as None (default)
        ..Default::default()
    });

    // Connect to the push server
    connect(&pc)
        .await
        .map_err(|e| format!("connect failed: {}", e))?;

    // Subscribe to market data for multiple symbols
    println!("\n=== Subscribing to market data ===");
    pc.subscribe(&SubjectType::Quote, Some("AAPL,TSLA,GOOG"), None, None);
    pc.subscribe(&SubjectType::Tick, Some("AAPL"), None, None);
    pc.subscribe(&SubjectType::Depth, Some("AAPL"), None, None);
    pc.subscribe(&SubjectType::Kline, Some("AAPL"), None, None);
    pc.subscribe(&SubjectType::Option, Some("AAPL"), None, None);
    pc.subscribe(&SubjectType::Future, Some("CL2506"), None, None);
    pc.subscribe(&SubjectType::StockTop, None, None, Some("US"));

    // Subscribe to account-level push data
    println!("=== Subscribing to account updates ===");
    pc.subscribe(&SubjectType::Asset, None, Some(&account), None);
    pc.subscribe(&SubjectType::Position, None, Some(&account), None);
    pc.subscribe(&SubjectType::Order, None, Some(&account), None);
    pc.subscribe(&SubjectType::Transaction, None, Some(&account), None);

    println!("All subscriptions active. Press Ctrl+C to exit.\n");
    tokio::signal::ctrl_c().await?;

    // Clean up
    pc.disconnect();
    println!("Disconnected.");
    Ok(())
}
