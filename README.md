# Tiger OpenAPI Rust SDK

Rust SDK for Tiger Brokers OpenAPI. Provides market data queries, order execution, account management, and real-time push notifications.

[![Crates.io](https://img.shields.io/crates/v/tigeropen.svg)](https://crates.io/crates/tigeropen)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
tigeropen = "0.5.13"
tokio = { version = "1", features = ["full"] }
```

Requires Rust 1.88 or later. Building the SDK also requires the Protocol Buffers compiler (`protoc`) on your `PATH`.

## Quick Start

```rust
use tigeropen::config::ClientConfig;
use tigeropen::quote::QuoteClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load config from properties file
    let config = ClientConfig::builder()
        .properties_file("tiger_openapi_config.properties")
        .build()?;

    // 2. Create a quote client and query market data
    let qc = QuoteClient::from_config(config);
    let states = qc.get_market_state("US").await?;
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
use tigeropen::model::{BriefRequest, KlineRequest, OptionChainItem, OptionChainRequest, QuoteDepthRequest, TradeTickRequest};
use tigeropen::quote::QuoteClient;

let qc = QuoteClient::from_config(config);

// Market state
let states = qc.get_market_state("US").await?;

// Real-time quotes
let quotes = qc.get_real_time_quote(BriefRequest {
    symbols: Some(vec!["AAPL".into(), "TSLA".into()]),
    ..Default::default()
}).await?;

// K-line data
let klines = qc.get_kline(KlineRequest {
    symbols: Some(vec!["AAPL".into()]),
    period: Some("day".into()),
    ..Default::default()
}).await?;

// Timeline
let timeline = qc.get_timeline(&["AAPL"]).await?;

// Depth quotes
let depth = qc.get_quote_depth(QuoteDepthRequest {
    symbols: Some(vec!["AAPL".into()]),
    market: Some("US".into()),
    ..Default::default()
}).await?;

// Option expiration dates
let expirations = qc.get_option_expiration(&["AAPL"], Some("US")).await?;

// Option chain
if let Some(expiry) = expirations.first().and_then(|item| item.timestamps.first()) {
    let chain = qc.get_option_chain(OptionChainRequest::new(vec![
        OptionChainItem::new("AAPL", *expiry),
    ])).await?;
}

// Futures exchange list
let exchanges = qc.get_future_exchange().await?;
```

## News

The SDK exposes two news query APIs. Both are routed to the **main gateway** (`server_url`, same as trade requests), not the quote server:

| API | method | Description |
|-----|--------|-------------|
| Stock news | `news_stock_list` | News list for a single symbol, paged by `page_count` / `page_size` |
| Market highlights | `news_highlight_list` | Market highlights, paged by the `last_id` cursor |

### Stock News

```rust
use tigeropen::model::enums::Language;
use tigeropen::model::news_requests::NewsStockListRequest;
use tigeropen::news::NewsClient;

let nc = NewsClient::from_config(config);

let page = nc
    .get_stock_news(NewsStockListRequest {
        symbol: "AAPL".to_string(),
        lang: Some(Language::ZhCn),
        page_count: Some(1),
        page_size: Some(20),
    })
    .await?;
for item in &page.items {
    println!("{} — {}", item.title, item.url);
}
```

Parameters:

| Parameter | Type | Required | Description |
|-----------|------|:--------:|-------------|
| `symbol` | `String` | Yes | Stock symbol. **Single symbol only** — batch queries are not supported by the downstream news service |
| `lang` | `Option<Language>` | No | `zh_CN` / `zh_TW` / `en_US` |
| `page_count` | `Option<i32>` | No | Page number, default 1 (server side), must be >= 1 |
| `page_size` | `Option<i32>` | No | Page size, default 20 (server side), range 1–50 |

### Market Highlights

```rust
use tigeropen::model::news_requests::NewsHighlightListRequest;

let page = nc
    .get_market_news(NewsHighlightListRequest {
        lang: Some(Language::EnUs),
        ..Default::default()
    })
    .await?;
// Next page: pass the id of the last item of this page as last_id
let next_last_id = page.items.last().map(|item| item.id.clone());
let next_page = nc
    .get_market_news(NewsHighlightListRequest {
        last_id: next_last_id,
        ..Default::default()
    })
    .await?;
```

Parameters:

| Parameter | Type | Required | Description |
|-----------|------|:--------:|-------------|
| `lang` | `Option<Language>` | No | Same as above |
| `last_id` | `Option<String>` | No | Cursor — the **only** pagination path for this API. `None` starts from the newest; pass the `id` of the last item of the current page to get the next page |

### Response Structure

Both methods return a `NewsPage` (wire fields are camelCase):

| Field | Type | Description |
|-------|------|-------------|
| `items` | `Vec<NewsItem>` | News items on the current page (may be empty) |
| `page_count` | `i32` | Current page number (always 1 for market highlights) |
| `page_size` | `i32` | Page size |
| `total_page` | `i32` | Total number of pages |
| `total_size` | `i32` | Total number of items |

Each `NewsItem`:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Article ID (source of the `last_id` cursor) |
| `title` | `String` | Title |
| `summary` | `String` | Summary |
| `pub_timestamp` | `i64` | Publish timestamp (seconds) |
| `url` | `String` | Detail page URL |

## Trading

```rust
use tigeropen::model::{limit_order, AssetsRequest, OrdersRequest, PositionsRequest};
use tigeropen::trade::TradeClient;

let account = config.account.clone();
let tc = TradeClient::from_config(config);
let order = limit_order(&account, "AAPL", "STK", "BUY", 1, 150.0);

// Preview the order without placing it
let preview = tc.preview_order(order.clone()).await?;

// Place a limit order, then use its global ID to modify and cancel it
if let Some(placed) = tc.place_order(order.clone()).await? {
    let mut modified = order;
    modified.limit_price = Some(155.0);
    tc.modify_order(placed.id, modified).await?;
    tc.cancel_order(placed.id).await?;
}

// Query orders, positions, and assets
let orders = tc.get_orders(OrdersRequest::default()).await?;
let positions = tc.get_positions(PositionsRequest::default()).await?;
let assets = tc.get_assets(AssetsRequest::default()).await?;
```

## Generic API Call (execute)

When the SDK hasn't wrapped a specific API yet, use `HttpClient::execute` directly:

```rust
use tigeropen::client::http_client::HttpClient;

let http = HttpClient::with_quote_server(config);
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

    // Asset / Position push with a specific quote type ("ETH" / "RTH" / "OVERNIGHT")
    pc.subscribe_with_asset_quote_type(
        &SubjectType::Position,
        None,
        Some(&account),
        None,
        Some("OVERNIGHT"),
    );

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
│   ├── news/      # News query client (main gateway)
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

- [Tiger OpenAPI Documentation](https://docs-en.itigerup.com/docs/)
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
tigeropen = "0.5.13"
tokio = { version = "1", features = ["full"] }
```

要求 Rust 1.88 或更高版本。构建 SDK 时还需要安装 Protocol Buffers 编译器（`protoc`），并将其加入 `PATH`。

## 快速开始

```rust
use tigeropen::config::ClientConfig;
use tigeropen::quote::QuoteClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 从 properties 文件加载配置
    let config = ClientConfig::builder()
        .properties_file("tiger_openapi_config.properties")
        .build()?;

    // 2. 创建行情客户端并查询市场状态
    let qc = QuoteClient::from_config(config);
    let states = qc.get_market_state("US").await?;
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
use tigeropen::model::{BriefRequest, KlineRequest, OptionChainItem, OptionChainRequest, QuoteDepthRequest, TradeTickRequest};
use tigeropen::quote::QuoteClient;

let qc = QuoteClient::from_config(config);

// 市场状态
let states = qc.get_market_state("US").await?;

// 实时报价
let quotes = qc.get_real_time_quote(BriefRequest {
    symbols: Some(vec!["AAPL".into(), "TSLA".into()]),
    ..Default::default()
}).await?;

// K 线数据
let klines = qc.get_kline(KlineRequest {
    symbols: Some(vec!["AAPL".into()]),
    period: Some("day".into()),
    ..Default::default()
}).await?;

// 分时数据
let timeline = qc.get_timeline(&["AAPL"]).await?;

// 深度行情
let depth = qc.get_quote_depth(QuoteDepthRequest {
    symbols: Some(vec!["AAPL".into()]),
    market: Some("US".into()),
    ..Default::default()
}).await?;

// 期权到期日
let expirations = qc.get_option_expiration(&["AAPL"], Some("US")).await?;

// 期权链
if let Some(expiry) = expirations.first().and_then(|item| item.timestamps.first()) {
    let chain = qc.get_option_chain(OptionChainRequest::new(vec![
        OptionChainItem::new("AAPL", *expiry),
    ])).await?;
}

// 期货交易所列表
let exchanges = qc.get_future_exchange().await?;

// 逐笔成交
let ticks = qc.get_trade_tick(TradeTickRequest { symbols: Some(vec!["AAPL".into()]), ..Default::default() }).await?;
// ticks[0].items[i].part_code  — 交易所短名，如 "NYSE"、"NSDQ"
// ticks[0].items[i].part_name  — 交易所全名，如 "New York Stock Exchange, LLC (NYSE)"
// ticks[0].items[i].cond       — 已转换的成交条件，如 "US_REGULAR_SALE"，参见下方说明
```

### 逐笔成交 `cond` 字段说明

`trade_tick` 接口和 push 层返回的 `cond` 字段已由 SDK 转换为可读字符串，含义如下：

**美股（US）**

| 字符 | 含义 |
|------|------|
| ` `（空格）| 常规交易（Regular Sale） |
| `B` | 批量交易（Bunched Trade） |
| `C` | 现金交易（Cash Trade） |
| `F` | 跨市场交易（Intermarket Sweep） |
| `G` | 批量卖出（Bunched Sold Trade） |
| `H` | 离价交易（Price Variation Trade） |
| `I` | 碎股交易（Odd Lot Trade） |
| `K` | 纽交所第 127/155 条交易 |
| `L` | 延迟交易（Sold Last） |
| `M` | 中央收市价（Market Center Close Price） |
| `N` | 隔日交易（Next Day Trade） |
| `O` | 中央开盘价交易（Market Center Opening Trade） |
| `P` | 前参考价（Prior Reference Price） |
| `Q` | 中央开盘价（Market Center Open Price） |
| `R` | 卖方（Seller） |
| `T` | 盘前盘后交易（Form T） |
| `U` | 延长交易时段（Extended Trading Hours） |
| `V` | 合单交易（Contingent Trade） |
| `W` | 均价交易（Average Price Trade） |
| `X` | 跨市场交易（Cross Trade） |
| `Z` | 场外售出（Sold Out of Sequence） |
| `4` | 衍生工具定价（Derivatively Priced） |
| `7` | 合单交易（Qualified Contingent Trade） |

**港股（HK）**

| 字符 | 含义 |
|------|------|
| ` `（空格）| 自动对盘（Automatch Normal） |
| `D` | 碎股交易（Odd Lot Trade） |
| `U` | 竞价交易（Auction Trade） |
| `*` | 场外交易（Overseas Trade） |
| `P` | 开市前成交（Late Trade Off Exchange） |
| `M` | 非自动对盘（Non-Direct Off Exchange Trade） |
| `X` | 同券商自动对盘（Direct Off Exchange Trade） |
| `Y` | 同券商非自动对盘（Automatic Internalized） |

如上表所示，`cond` 值因市场（美股/港股）不同而有所区别。

## 资讯查询

SDK 提供两个资讯查询接口，均路由到**主网关**（`server_url`，与交易请求同款），不走行情域名：

| 接口 | method | 说明 |
|------|--------|------|
| 个股新闻 | `news_stock_list` | 查询单只股票的资讯列表，按 `page_count` / `page_size` 翻页 |
| 市场要闻 | `news_highlight_list` | 查询市场要闻列表，按 `last_id` 游标翻页 |

### 个股新闻

```rust
use tigeropen::model::enums::Language;
use tigeropen::model::news_requests::NewsStockListRequest;
use tigeropen::news::NewsClient;

let nc = NewsClient::from_config(config);

let page = nc
    .get_stock_news(NewsStockListRequest {
        symbol: "AAPL".to_string(),
        lang: Some(Language::ZhCn),
        page_count: Some(1),
        page_size: Some(20),
    })
    .await?;
for item in &page.items {
    println!("{} — {}", item.title, item.url);
}
```

参数说明：

| 参数 | 类型 | 必填 | 说明 |
|------|------|:----:|------|
| `symbol` | `String` | 是 | 股票代码，**仅支持单只**（该接口不支持批量查询，空值由服务端报参数错误） |
| `lang` | `Option<Language>` | 否 | 语言枚举：`zh_CN` / `zh_TW` / `en_US` |
| `page_count` | `Option<i32>` | 否 | 页码，缺省 = 服务端默认 1，取值 >= 1 |
| `page_size` | `Option<i32>` | 否 | 每页条数，缺省 = 服务端默认 20，取值 1–50 |

### 市场要闻

```rust
use tigeropen::model::news_requests::NewsHighlightListRequest;

let page = nc
    .get_market_news(NewsHighlightListRequest {
        lang: Some(Language::EnUs),
        ..Default::default()
    })
    .await?;
// 下一页：把本页最后一条的 id 作为 last_id 传回
let next_last_id = page.items.last().map(|item| item.id.clone());
let next_page = nc
    .get_market_news(NewsHighlightListRequest {
        last_id: next_last_id,
        ..Default::default()
    })
    .await?;
```

参数说明：

| 参数 | 类型 | 必填 | 说明 |
|------|------|:----:|------|
| `lang` | `Option<Language>` | 否 | 同上 |
| `last_id` | `Option<String>` | 否 | 游标，该接口**唯一**翻页方式。不传从最新开始；传上一页 `items` 最后一条的 `id` 获取下一页 |

### 响应结构

两个接口的响应均为 `NewsPage`（wire 出参驼峰，SDK 字段为 snake_case）：

| 字段 | 类型 | 说明 |
|------|------|------|
| `items` | `Vec<NewsItem>` | 当前页资讯列表（可为空数组） |
| `page_count` | `i32` | 当前页码（要闻接口恒为 1） |
| `page_size` | `i32` | 每页条数 |
| `total_page` | `i32` | 总页数 |
| `total_size` | `i32` | 总条数 |

每条 `NewsItem`：

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 文章 ID（要闻游标翻页 `last_id` 的取值来源） |
| `title` | `String` | 标题 |
| `summary` | `String` | 摘要 |
| `pub_timestamp` | `i64` | 发布时间戳（秒级） |
| `url` | `String` | 详情页 URL |

## 交易操作

```rust
use tigeropen::model::{limit_order, AssetsRequest, OrdersRequest, PositionsRequest};
use tigeropen::trade::TradeClient;

let account = config.account.clone();
let tc = TradeClient::from_config(config);
let order = limit_order(&account, "AAPL", "STK", "BUY", 1, 150.0);

// 预览订单（不实际下单）
let preview = tc.preview_order(order.clone()).await?;

// 下限价单，使用返回的全局订单 ID 修改和取消
if let Some(placed) = tc.place_order(order.clone()).await? {
    let mut modified = order;
    modified.limit_price = Some(155.0);
    tc.modify_order(placed.id, modified).await?;
    tc.cancel_order(placed.id).await?;
}

// 查询订单、持仓、资产
let orders = tc.get_orders(OrdersRequest::default()).await?;
let positions = tc.get_positions(PositionsRequest::default()).await?;
let assets = tc.get_assets(AssetsRequest::default()).await?;
```

## 通用方法（execute）

当 SDK 尚未封装某个 API 时，可以使用 `HttpClient::execute` 直接调用：

```rust
use tigeropen::client::http_client::HttpClient;

let http = HttpClient::with_quote_server(config);
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

    // 资产 / 持仓推送可指定行情口径（"ETH" / "RTH" / "OVERNIGHT"）
    pc.subscribe_with_asset_quote_type(
        &SubjectType::Position,
        None,
        Some(&account),
        None,
        Some("OVERNIGHT"),
    );

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
│   ├── news/      # 资讯查询客户端（主网关）
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

- [老虎证券 OpenAPI 文档](https://docs.itigerup.com/docs/)
- [crates.io 包主页](https://crates.io/crates/tigeropen)
- [docs.rs 文档](https://docs.rs/tigeropen)

## 许可证

[MIT License](LICENSE)
