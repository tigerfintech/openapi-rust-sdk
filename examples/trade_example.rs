//! 交易下单示例
//!
//! 演示如何使用 TradeClient 进行下单和查询。

/*
use tigeropen::config::ClientConfig;
use tigeropen::trade::TradeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClientConfig::builder()
        .tiger_id("你的 tiger_id")
        .private_key("你的 RSA 私钥")
        .account("你的交易账户")
        .build()?;

    let tc = TradeClient::new(config);

    // 查询订单
    let orders = tc.get_orders().await?;
    println!("订单列表: {:?}", orders);

    // 查询持仓
    let positions = tc.get_positions().await?;
    println!("持仓列表: {:?}", positions);

    Ok(())
}
*/

fn main() {
    println!("请取消注释上方代码并填入真实配置后运行");
}
