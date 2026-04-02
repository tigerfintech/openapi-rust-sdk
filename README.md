# Tiger OpenAPI Rust SDK

老虎证券 OpenAPI 的 Rust SDK，提供行情查询、交易下单、账户管理和实时推送等功能。

[![Crates.io](https://img.shields.io/crates/v/tigeropen.svg)](https://crates.io/crates/tigeropen)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
tigeropen = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

要求 Rust 1.70 或更高版本。

## Quick Start

以下是一个完整的示例，从配置到查询行情：

```rust
use tigeropen::config::ClientConfig;
use tigeropen::client::http_client::HttpClient;
use tigeropen::quote::QuoteClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建配置（从 properties 文件加载）
    let config = ClientConfig::builder()
        .properties_file("tiger_openapi_config.properties")
        .build()?;

    // 2. 创建 HTTP 客户端
    let http_client = HttpClient::new(config.clone());

    // 3. 创建行情客户端并查询
    let qc = QuoteClient::new(&http_client);
    let states = qc.get_market_state("US").await?;
    println!("美股市场状态: {:?}", states);

    Ok(())
}
```

## 配置

SDK 支持三种配置方式，优先级：**环境变量 > Builder 设置（含配置文件） > 默认值**。

### 方式一：Builder 模式

```rust
use tigeropen::config::ClientConfig;

let config = ClientConfig::builder()
    .tiger_id("你的 tiger_id")
    .private_key("你的 RSA 私钥")
    .account("你的交易账户")
    .build()?;
```

### 方式二：从 properties 配置文件加载

```rust
let config = ClientConfig::builder()
    .properties_file("tiger_openapi_config.properties")
    .build()?;
```

配置文件格式：

```properties
tiger_id=你的开发者ID
private_key=你的RSA私钥
account=你的交易账户
```

### 方式三：环境变量

```bash
export TIGEROPEN_TIGER_ID=你的开发者ID
export TIGEROPEN_PRIVATE_KEY=你的RSA私钥
export TIGEROPEN_ACCOUNT=你的交易账户
```

### 配置项说明

| 配置项 | 说明 | 必填 | 默认值 |
|--------|------|------|--------|
| tiger_id | 开发者 ID | 是 | - |
| private_key | RSA 私钥 | 是 | - |
| account | 交易账户 | 否 | - |
| language | 语言（ZhCn/EnUs） | 否 | ZhCn |
| timeout | 请求超时 | 否 | 15s |
| sandbox_debug | 是否使用沙箱环境 | 否 | false |

## 行情查询

```rust
use tigeropen::client::http_client::HttpClient;
use tigeropen::quote::QuoteClient;

let http_client = HttpClient::new(config.clone());
let qc = QuoteClient::new(&http_client);

// 获取市场状态
let states = qc.get_market_state("US").await?;

// 获取实时报价
let briefs = qc.get_brief(&["AAPL", "TSLA"]).await?;

// 获取 K 线数据
let klines = qc.get_kline("AAPL", "day").await?;

// 获取分时数据
let timeline = qc.get_timeline(&["AAPL"]).await?;

// 获取深度行情
let depth = qc.get_quote_depth("AAPL").await?;

// 获取期权到期日
let expiry = qc.get_option_expiration("AAPL").await?;

// 获取期权链
let chain = qc.get_option_chain("AAPL", "2024-01-19").await?;

// 获取期货交易所列表
let exchanges = qc.get_future_exchange().await?;
```

## 交易操作

```rust
use tigeropen::client::http_client::HttpClient;
use tigeropen::trade::TradeClient;
use serde_json::json;

let http_client = HttpClient::new(config.clone());
let tc = TradeClient::new(&http_client, &config.account);

// 构造限价单
let order = json!({
    "symbol": "AAPL",
    "secType": "STK",
    "action": "BUY",
    "orderType": "LMT",
    "totalQuantity": 100,
    "limitPrice": 150.0,
    "timeInForce": "DAY",
});

// 下单
let result = tc.place_order(order.clone()).await?;

// 预览订单（不实际下单）
let preview = tc.preview_order(order.clone()).await?;

// 修改订单
let modified = json!({
    "symbol": "AAPL",
    "secType": "STK",
    "action": "BUY",
    "orderType": "LMT",
    "totalQuantity": 100,
    "limitPrice": 155.0,
});
tc.modify_order(order_id, modified).await?;

// 取消订单
tc.cancel_order(order_id).await?;

// 查询全部订单
let orders = tc.get_orders().await?;

// 查询持仓
let positions = tc.get_positions().await?;

// 查询资产
let assets = tc.get_assets().await?;
```

## 通用方法（execute）

当 SDK 尚未封装某个 API 时，可以使用 `HttpClient::execute` 直接调用：

```rust
let http_client = HttpClient::new(config.clone());

// 直接传入 API 方法名和 JSON 参数
let resp = http_client.execute("market_state", r#"{"market":"US"}"#).await?;
println!("原始响应: {}", resp);
```

## 实时推送

通过 WebSocket 长连接接收实时行情和账户推送，支持自动重连和心跳保活：

```rust
use tigeropen::push::{PushClient, Callbacks, QuotePushData, SubjectType};
use std::sync::Arc;

let pc = PushClient::new(config.clone(), None);

// 设置回调
pc.set_callbacks(Callbacks {
    on_quote: Some(Arc::new(|data: QuotePushData| {
        println!("行情推送: {} 最新价: {:?}", data.symbol, data.latest_price);
    })),
    on_order: Some(Arc::new(|data| {
        println!("订单推送: {:?}", data);
    })),
    on_asset: Some(Arc::new(|data| {
        println!("资产变动推送: {:?}", data);
    })),
    on_connect: Some(Arc::new(|| {
        println!("推送连接成功");
    })),
    on_disconnect: Some(Arc::new(|| {
        println!("推送连接断开");
    })),
    on_error: Some(Arc::new(|err| {
        eprintln!("推送错误: {}", err);
    })),
    ..Default::default()
});

// 订阅行情
pc.add_subscription(SubjectType::Quote, &["AAPL".into(), "TSLA".into()]);

// 订阅账户推送
pc.add_account_sub(SubjectType::Asset);
pc.add_account_sub(SubjectType::Order);
pc.add_account_sub(SubjectType::Position);

// 断开连接
pc.disconnect();
```

## 项目结构

```
openapi-rust-sdk/
├── src/
│   ├── config/    # 配置管理（ClientConfig Builder、ConfigParser、动态域名）
│   ├── signer/    # RSA 签名
│   ├── client/    # HTTP 客户端（请求/响应、重试策略、execute）
│   ├── model/     # 数据模型（Order、Contract、Position、枚举）
│   ├── quote/     # 行情查询客户端
│   ├── trade/     # 交易客户端
│   ├── push/      # WebSocket 实时推送客户端
│   ├── error.rs   # 错误类型
│   ├── logger.rs  # 日志模块
│   └── lib.rs     # 统一导出
├── examples/      # 示例代码
└── tests/         # 测试
```

## API 参考

- [老虎证券 OpenAPI 文档](https://quant.itigerup.com/openapi/zh/python/overview/introduction.html)
- [crates.io 包主页](https://crates.io/crates/tigeropen)
- [docs.rs 文档](https://docs.rs/tigeropen)

## 许可证

[MIT License](LICENSE)
