//! 实时推送示例
//!
//! 演示如何使用 PushClient 接收实时行情推送。

/*
use tigeropen::config::ClientConfig;
use tigeropen::push::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClientConfig::builder()
        .properties_file("tiger_openapi_config.properties")
        .build()?;

    let pc = PushClient::new(config, None);

    pc.set_callbacks(Callbacks {
        on_quote: Some(Arc::new(|data| {
            println!("[行情] {}: {:?}", data.symbol, data.latest_price);
        })),
        on_connect: Some(Arc::new(|| println!("已连接"))),
        on_disconnect: Some(Arc::new(|| println!("已断开"))),
        ..Default::default()
    });

    pc.connect().await?;
    pc.add_subscription(SubjectType::Quote, &["AAPL".into()]);

    tokio::signal::ctrl_c().await?;
    pc.disconnect();
    Ok(())
}
*/

fn main() {
    println!("请取消注释上方代码并填入真实配置后运行");
}
