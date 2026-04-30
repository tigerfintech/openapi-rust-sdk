# Tiger OpenAPI Rust SDK

Rust SDK for Tiger Brokers OpenAPI. Provides market data queries, order execution, account management, and real-time push notifications.

[![Crates.io](https://img.shields.io/crates/v/tigeropen.svg)](https://crates.io/crates/tigeropen)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
tigeropen = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

Requires Rust 1.70 or later.

## Quick Start

```rust
use tigeropen::config::ClientConfig;
use tigeropen::client::http_client::HttpClient;
use tigeropen::quote::QuoteClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load config from properties file
    let config = ClientConfig::builder()
        .properties_file("tiger_openapi_config.properties")
        .build()?;

    // 2. Create HTTP client
    let http = HttpClient::new(config);

    // 3. Query market data
    let qc = QuoteClient::new(&http);
    let states = qc.market_state("US").await?;
    println!("US market state: {:?}", states);

    Ok(())
}
```

## Configuration

The SDK supports multiple configuration methods. Priority: **environment variables > builder setters (incl. properties file) > auto-discovered config file > defaults**.

### Method 1: Properties File

The most common approach. Create a `tiger_openapi_config.properties` file:

```properties
tiger_id=your_developer_id
private_key=your_rsa_private_key
account=your_trading_account
```

Load it explicitly:

```rust
let config = ClientConfig::builder()
    .properties_file("tiger_openapi_config.properties")
    .build()?;
```

### Method 2: Auto-Discovery

If you call `build()` without setting `tiger_id` or `private_key`, the builder automatically searches for a config file in this order:

1. `./tiger_openapi_config.properties` (current directory)
2. `~/.tigeropen/tiger_openapi_config.properties` (home directory)

This means you can simply do:

```rust
let config = ClientConfig::builder().build()?; // auto-discovers config
```

Place your config file at `~/.tigeropen/tiger_openapi_config.properties` and it will be picked up automatically across all your projects.

### Method 3: Builder

Set values programmatically:

```rust
let config = ClientConfig::builder()
    .tiger_id("your_developer_id")
    .private_key("your_rsa_private_key")
    .account("your_trading_account")
    .build()?;
```

### Method 4: Environment Variables

```bash
export TIGEROPEN_TIGER_ID=your_developer_id
export TIGEROPEN_PRIVATE_KEY=your_rsa_private_key
export TIGEROPEN_ACCOUNT=your_trading_account
```

Environment variables have the highest priority and override all other methods.

### Configuration Reference

| Field | Description | Required | Default |
|-------|-------------|----------|---------|
| tiger_id | Developer ID | Yes | - |
| private_key | RSA private key | Yes | - |
| account | Trading account | No | - |
| language | Language (ZhCn/EnUs) | No | ZhCn |
| timeout | Request timeout | No | 15s |

## Market Data

```rust
use tigeropen::client::http_client::HttpClient;
use tigeropen::quote::QuoteClient;

let http = HttpClient::new(config);
let qc = QuoteClient::new(&http);

// Market state
let states = qc.market_state("US").await?;

// Real-time quotes
let quotes = qc.quote_real_time(&["AAPL", "TSLA"]).await?;

// K-line data
let klines = qc.kline("AAPL", "day").await?;

// Timeline
let timeline = qc.timeline(&["AAPL"]).await?;

// Depth quotes
let depth = qc.quote_depth("AAPL").await?;

// Option expiration dates
let expiry = qc.option_expiration("AAPL").await?;

// Option chain
let chain = qc.option_chain("AAPL", "2024-01-19").await?;

// Futures exchange list
let exchanges = qc.future_exchange().await?;
```

## Trading

```rust
use tigeropen::client::http_client::HttpClient;
use tigeropen::trade::TradeClient;
use serde_json::json;

let http = HttpClient::new(config);
let tc = TradeClient::new(&http, "your_account");

// Place a limit order
let order = json!({
    "symbol": "AAPL",
    "secType": "STK",
    "action": "BUY",
    "orderType": "LMT",
    "totalQuantity": 100,
    "limitPrice": 150.0,
    "timeInForce": "DAY",
});
let result = tc.place_order(order).await?;

// Preview order (no actual execution)
let preview = tc.preview_order(order).await?;

// Modify order
tc.modify_order(order_id, json!({"limitPrice": 155.0})).await?;

// Cancel order
tc.cancel_order(order_id).await?;

// Query orders, positions, assets
let orders = tc.orders().await?;
let positions = tc.positions().await?;
let assets = tc.assets().await?;
```

## Generic API Call (execute)

When the SDK hasn't wrapped a specific API yet, use `HttpClient::execute` directly:

```rust
let http = HttpClient::new(config);
let resp = http.execute("market_state", r#"{"market":"US"}"#).await?;
println!("Raw response: {}", resp);
```

## Real-Time Push

The push client uses a **TCP + TLS + Protobuf** persistent connection for real-time market data and account notifications. It supports automatic reconnection and heartbeat keep-alive.

Callback parameters use Protobuf-generated types (`pb::QuoteData`, `pb::OrderStatusData`, `pb::AssetData`, etc.).

```rust
use std::sync::Arc;
use tigeropen::config::ClientConfig;
use tigeropen::push::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClientConfig::builder().build()?; // auto-discovers config
    let account = config.account.clone();

    let pc = Arc::new(PushClient::new(config, None));

    // Set callbacks (parameters are pb::* Protobuf types)
    pc.set_callbacks(Callbacks {
        on_quote: Some(Arc::new(|data: pb::QuoteData| {
            println!("[Quote] {} price={:?} vol={:?}", data.symbol, data.latest_price, data.volume);
        })),
        on_order: Some(Arc::new(|data: pb::OrderStatusData| {
            println!("[Order] {:?}", data);
        })),
        on_asset: Some(Arc::new(|data: pb::AssetData| {
            println!("[Asset] {:?}", data);
        })),
        on_position: Some(Arc::new(|data: pb::PositionData| {
            println!("[Position] {:?}", data);
        })),
        on_connect: Some(Arc::new(|| println!("Connected"))),
        on_disconnect: Some(Arc::new(|| println!("Disconnected"))),
        on_error: Some(Arc::new(|msg| eprintln!("Error: {}", msg))),
        ..Default::default()
    });

    // Connect using the free function push::connect()
    connect(&pc).await.map_err(|e| format!("connect failed: {}", e))?;

    // Subscribe to market data
    pc.subscribe(&SubjectType::Quote, Some("AAPL,TSLA"), None, None);

    // Subscribe to account push
    pc.subscribe(&SubjectType::Asset, None, Some(&account), None);
    pc.subscribe(&SubjectType::Order, None, Some(&account), None);

    println!("Subscribed. Press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await?;
    pc.disconnect();
    Ok(())
}
```

## Project Structure

```
openapi-rust-sdk/
├── src/
│   ├── config/    # Configuration (ClientConfig builder, config parser, dynamic domain)
│   ├── signer/    # RSA signing
│   ├── client/    # HTTP client (request/response, retry, execute)
│   ├── model/     # Data models (Order, Contract, Position, enums)
│   ├── quote/     # Market data query client
│   ├── trade/     # Trading client
│   ├── push/      # TCP+TLS push client (Protobuf binary protocol)
│   ├── error.rs   # Error types
│   ├── logger.rs  # Logging
│   └── lib.rs     # Public exports
├── examples/      # Example code
└── tests/         # Tests
```

## API Reference

- [Tiger OpenAPI Documentation](https://quant.itigerup.com/openapi/zh/python/overview/introduction.html)
- [crates.io](https://crates.io/crates/tigeropen)
- [docs.rs](https://docs.rs/tigeropen)

## License

[MIT License](LICENSE)

---

# Tiger OpenAPI Rust SDK（中文）

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

## 快速开始

```rust
use tigeropen::config::ClientConfig;
use tigeropen::client::http_client::HttpClient;
use tigeropen::quote::QuoteClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 从 properties 文件加载配置
    let config = ClientConfig::builder()
        .properties_file("tiger_openapi_config.properties")
        .build()?;

    // 2. 创建 HTTP 客户端
    let http = HttpClient::new(config);

    // 3. 查询行情
    let qc = QuoteClient::new(&http);
    let states = qc.market_state("US").await?;
    println!("美股市场状态: {:?}", states);

    Ok(())
}
```

## 配置

SDK 支持多种配置方式，优先级：**环境变量 > Builder 设置（含配置文件） > 自动发现的配置文件 > 默认值**。

### 方式一：Properties 配置文件

最常用的方式。创建 `tiger_openapi_config.properties` 文件：

```properties
tiger_id=你的开发者ID
private_key=你的RSA私钥
account=你的交易账户
```

显式加载：

```rust
let config = ClientConfig::builder()
    .properties_file("tiger_openapi_config.properties")
    .build()?;
```

### 方式二：自动发现

如果调用 `build()` 时未设置 `tiger_id` 或 `private_key`，Builder 会按以下顺序自动搜索配置文件：

1. `./tiger_openapi_config.properties`（当前目录）
2. `~/.tigeropen/tiger_openapi_config.properties`（用户主目录）

因此你可以直接：

```rust
let config = ClientConfig::builder().build()?; // 自动发现配置
```

将配置文件放在 `~/.tigeropen/tiger_openapi_config.properties`，所有项目都能自动加载。

### 方式三：Builder 模式

通过代码设置：

```rust
let config = ClientConfig::builder()
    .tiger_id("你的开发者ID")
    .private_key("你的RSA私钥")
    .account("你的交易账户")
    .build()?;
```

### 方式四：环境变量

```bash
export TIGEROPEN_TIGER_ID=你的开发者ID
export TIGEROPEN_PRIVATE_KEY=你的RSA私钥
export TIGEROPEN_ACCOUNT=你的交易账户
```

环境变量优先级最高，会覆盖所有其他配置方式。

### 配置项说明

| 配置项 | 说明 | 必填 | 默认值 |
|--------|------|------|--------|
| tiger_id | 开发者 ID | 是 | - |
| private_key | RSA 私钥 | 是 | - |
| account | 交易账户 | 否 | - |
| language | 语言（ZhCn/EnUs） | 否 | ZhCn |
| timeout | 请求超时 | 否 | 15s |

## 行情查询

```rust
use tigeropen::client::http_client::HttpClient;
use tigeropen::quote::QuoteClient;

let http = HttpClient::new(config);
let qc = QuoteClient::new(&http);

// 市场状态
let states = qc.market_state("US").await?;

// 实时报价
let quotes = qc.quote_real_time(&["AAPL", "TSLA"]).await?;

// K 线数据
let klines = qc.kline("AAPL", "day").await?;

// 分时数据
let timeline = qc.timeline(&["AAPL"]).await?;

// 深度行情
let depth = qc.quote_depth("AAPL").await?;

// 期权到期日
let expiry = qc.option_expiration("AAPL").await?;

// 期权链
let chain = qc.option_chain("AAPL", "2024-01-19").await?;

// 期货交易所列表
let exchanges = qc.future_exchange().await?;
```

## 交易操作

```rust
use tigeropen::client::http_client::HttpClient;
use tigeropen::trade::TradeClient;
use serde_json::json;

let http = HttpClient::new(config);
let tc = TradeClient::new(&http, "你的账户");

// 下限价单
let order = json!({
    "symbol": "AAPL",
    "secType": "STK",
    "action": "BUY",
    "orderType": "LMT",
    "totalQuantity": 100,
    "limitPrice": 150.0,
    "timeInForce": "DAY",
});
let result = tc.place_order(order).await?;

// 预览订单（不实际下单）
let preview = tc.preview_order(order).await?;

// 修改订单
tc.modify_order(order_id, json!({"limitPrice": 155.0})).await?;

// 取消订单
tc.cancel_order(order_id).await?;

// 查询订单、持仓、资产
let orders = tc.orders().await?;
let positions = tc.positions().await?;
let assets = tc.assets().await?;
```

## 通用方法（execute）

当 SDK 尚未封装某个 API 时，可以使用 `HttpClient::execute` 直接调用：

```rust
let http = HttpClient::new(config);
let resp = http.execute("market_state", r#"{"market":"US"}"#).await?;
println!("原始响应: {}", resp);
```

## 实时推送

推送客户端使用 **TCP + TLS + Protobuf** 长连接接收实时行情和账户推送通知，支持自动重连和心跳保活。

回调参数使用 Protobuf 生成的类型（`pb::QuoteData`、`pb::OrderStatusData`、`pb::AssetData` 等）。

```rust
use std::sync::Arc;
use tigeropen::config::ClientConfig;
use tigeropen::push::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClientConfig::builder().build()?; // 自动发现配置
    let account = config.account.clone();

    let pc = Arc::new(PushClient::new(config, None));

    // 设置回调（参数为 pb::* Protobuf 类型）
    pc.set_callbacks(Callbacks {
        on_quote: Some(Arc::new(|data: pb::QuoteData| {
            println!("[行情] {} 最新价={:?} 成交量={:?}", data.symbol, data.latest_price, data.volume);
        })),
        on_order: Some(Arc::new(|data: pb::OrderStatusData| {
            println!("[订单] {:?}", data);
        })),
        on_asset: Some(Arc::new(|data: pb::AssetData| {
            println!("[资产] {:?}", data);
        })),
        on_position: Some(Arc::new(|data: pb::PositionData| {
            println!("[持仓] {:?}", data);
        })),
        on_connect: Some(Arc::new(|| println!("推送连接成功"))),
        on_disconnect: Some(Arc::new(|| println!("推送连接断开"))),
        on_error: Some(Arc::new(|msg| eprintln!("推送错误: {}", msg))),
        ..Default::default()
    });

    // 使用 push::connect() 自由函数连接
    connect(&pc).await.map_err(|e| format!("连接失败: {}", e))?;

    // 订阅行情
    pc.subscribe(&SubjectType::Quote, Some("AAPL,TSLA"), None, None);

    // 订阅账户推送
    pc.subscribe(&SubjectType::Asset, None, Some(&account), None);
    pc.subscribe(&SubjectType::Order, None, Some(&account), None);

    println!("已订阅，按 Ctrl+C 退出");
    tokio::signal::ctrl_c().await?;
    pc.disconnect();
    Ok(())
}
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
│   ├── push/      # TCP+TLS 推送客户端（Protobuf 二进制协议）
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
