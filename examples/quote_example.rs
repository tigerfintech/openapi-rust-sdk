//! 行情查询示例
//!
//! 演示如何使用 QuoteClient 查询市场状态和实时报价。
//!
//! 运行方式：cargo run --example quote_example

/*
use tigeropen::config::ClientConfig;
use tigeropen::quote::QuoteClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置
    let config = ClientConfig::builder()
        .tiger_id("你的 tiger_id")
        .private_key("你的 RSA 私钥")
        .account("你的交易账户")
        .build()?;

    let qc = QuoteClient::new(config);

    // 查询市场状态
    println!("=== 市场状态 ===");
    let states = qc.get_market_state("US").await?;
    println!("{:?}", states);

    // 查询实时报价
    println!("\n=== 实时报价 ===");
    let quotes = qc.get_quote_real_time(&["AAPL", "TSLA"]).await?;
    println!("{:?}", quotes);

    Ok(())
}
*/

fn main() {
    println!("请取消注释上方代码并填入真实配置后运行");
}
