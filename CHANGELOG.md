# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.2] - 2026-07-08

### Added

- **`OptionChainItem` / `OptionChainRequest`**：`get_option_chain` 签名由 `&[(&str, &str)]` 改为 `OptionChainRequest`，`OptionChainItem` 支持三种构造方式：
  - `OptionChainItem::new(symbol, expiry_ms)` — 直接传毫秒时间戳
  - `OptionChainItem::from_date(symbol, "YYYY-MM-DD")?` — 日期字符串，按 symbol 自动推断时区（US → `America/New_York`，HK → `Asia/Hong_Kong`，其余 → `Asia/Shanghai`）
  - `OptionChainItem::from_date_tz(symbol, "YYYY-MM-DD", "America/New_York")?` — 显式指定时区

- **`OptionContractItem` / `OptionQuoteRequest`**：`get_option_quote` 签名由 `&[&str]`（OCC 字符串）改为 `OptionQuoteRequest`，`OptionContractItem` 支持：
  - `OptionContractItem::from_occ("AAPL 240119C00150000")?` — OCC 格式，按 symbol 推断时区
  - `OptionContractItem::from_occ_tz(occ, timezone)?` — 显式指定时区
  - `OptionContractItem::new(symbol, expiry_ms, right, strike)` — 直接构造

- **`OptionKlineItem` / `OptionKlineRequest`**：`get_option_kline` 签名由 `(&[&str], period)` 改为 `OptionKlineRequest`，`OptionKlineItem` 支持 `from_occ` / `from_occ_tz` / `new`，并可设置 `begin_time` / `end_time` / `limit`。

### Fixed

- **期权 expiry 时区错误**：原 `parse_expiry_to_ms` 将日期字符串转为 UTC 午夜时间戳，导致 US 期权 expiry 偏差 4–5 小时、HK 期权偏差 8 小时。现在按 symbol 推断交易所时区（与 Java SDK `SymbolUtil.getZoneIdBySymbol` 对齐）。

## [0.5.1] - 2026-07-07

### Deprecated

- **`get_brief` → `get_real_time_quote`**：方法名与 wire method `quote_real_time` 不一致，现以 `get_real_time_quote` 为主，旧名保留并标记 `#[deprecated(since = "0.5.1")]`。
- **`get_option_brief` → `get_option_quote`**：wire method 为 `option_brief`，更名为 `get_option_quote`。
- **`get_stock_delay_briefs` → `get_delayed_quote`**：wire method 为 `quote_delay`，更名与之对齐。
- **`get_warrant_briefs` → `get_warrant_quote`**：wire method 为 `warrant_briefs`，更名保持接口层风格一致。

## [0.5.0] - 2026-07-07

### Breaking Changes

- **`QuoteClient` / `TradeClient` 构造方式变更**：不再需要用户手动创建 `HttpClient`；新增 `from_config(config: ClientConfig)` 构造器，直接接受 `ClientConfig`，内部自动选择 trade/quote server。旧的 `new(http_client)` / `with_secret_key(http_client, ...)` 构造器继续可用，但参数从 `&HttpClient` 改为拥有所有权的 `HttpClient`（移除了 lifetime 参数 `<'a>`）。
- **`call_*` 系列方法改为 `pub`**：`call_into`、`call_into_versioned`、`call_into_items`、`call_into_list_or_object`、`call_optional`、`call_optional_versioned` 现在均为 `pub`，可直接用于自定义请求。
- **多 symbol 支持（行情接口签名变更）**：下列接口参数由单 symbol 改为 slice。调用方需更新：
  - `get_kline(symbol: &str, ...)` → `get_kline(symbols: &[&str], ...)`
  - `get_option_expiration(symbol: &str)` → `get_option_expiration(symbols: &[&str])`
  - `get_option_chain(symbol: &str, expiry: &str)` → `get_option_chain(items: &[(&str, &str)])`（每项为 `(symbol, expiry)` 对）
  - `get_option_kline(identifier: &str, period: &str)` → `get_option_kline(identifiers: &[&str], period: &str)`
  - `BarsByPageRequest.symbol: Option<String>` → `symbols: Option<Vec<String>>`
- **`get_kline` 签名变更（KlineRequest 结构体）**：`get_kline(symbols: &[&str], period: &str)` → `get_kline(req: KlineRequest)`；删除 `get_bars` 方法及 `BarsRequest` 类型（改名为 `KlineRequest`）。
- **`get_kline_by_page` 重命名**：`get_bars_by_page(req: BarsByPageRequest)` → `get_kline_by_page(req: KlineByPageRequest)`；删除 `BarsByPageRequest` 类型。
- **`get_option_bars` 删除**：改用 `get_option_kline`。
- **`get_future_bars` 删除**：改用 `get_future_kline(req: FutureKlineRequest)`；`FutureBarsRequest` 重命名为 `FutureKlineRequest`（字段相同，全部 `Option`）。
- **`get_future_kline_by_page` 重命名**：`get_future_bars_by_page(req: FutureBarsByPageRequest)` → `get_future_kline_by_page(req: FutureKlineByPageRequest)`。

## [0.4.4] - 2026-07-03

### Fixed

- **examples 未使用 `secret_key`**：`trade_example` 始终用 `TradeClient::new`，即使 config 中已加载 `secret_key`，机构账号下所有交易接口报 `access forbidden`；现在当 config 含 `secret_key` 时自动改用 `TradeClient::with_secret_key`

## [0.4.3] - 2026-06-24

### Added

- **冰山单支持**：新增 `iceberg_order()` 订单构造辅助函数，支持通过 `OrderRequest` 字段设置 `min_display_size`、`check_intervals`、`price_type`、`start_time`、`end_time` 等可选参数。
- **`Order` 结构体新增冰山单字段**：`display_size`、`min_display_size`、`check_intervals`、`price_type`、`start_time`、`end_time`。
- **`TradeClient::preview_order()`**：新增预览下单接口，接受任意 `Order`，返回 `Result<OrderPreviewResult>`。
- **单元测试**：`iceberg_order` 覆盖基础构造、可选参数、零值省略及序列化字段名五个场景。

### Deprecated

- **`get_brief` → `get_real_time_quote`**：方法名与 wire method `quote_real_time` 不一致，现以 `get_real_time_quote` 为主，旧名保留并标记 `#[deprecated(since = "0.5.0")]`。
- **`get_option_brief` → `get_option_quote`**：wire method 为 `option_brief`，更名为 `get_option_quote`。
- **`get_stock_delay_briefs` → `get_delayed_quote`**：wire method 为 `quote_delay`，更名与之对齐。
- **`get_warrant_briefs` → `get_warrant_quote`**：wire method 为 `warrant_briefs`，更名保持接口层风格一致。

## [0.4.2] - 2026-06-09

### Added

- **期权行权 5 个接口**：新增 `option_exercise_check`、`get_option_exercise_positions`、`submit_option_exercise`（返回 `Result<Option<bool>>`）、`get_option_exercise_records`、`cancel_option_exercise`（返回 `Result<Option<bool>>`）。
- **`TradeClient::with_secret_key()`**：机构账户可通过新构造器传入 `secret_key`，期权行权方法自动注入。
- **`ClientConfig::secret_key`**：支持从 `.properties` 文件读取 `secret_key`，通过 `ClientConfigBuilder::secret_key()` 显式设置。
- **`option_exercise_submit`/`option_exercise_cancel` 加入 `TRADE_OPERATIONS`**：非幂等行权写操作不参与自动重试。

### Fixed

- **`decode_value` 保留原始错误**：fallback 分支不再重复执行 `from_value` 调用，改为直接返回 `original_err`，错误信息不丢失。

## [0.4.1] - 2026-05-25

### Added

- **Token 自动刷新**：新增 `TokenManager`（`tokio::spawn` + oneshot channel 停止、`Drop` 自动清理）、`token_loader` / `token_writer` 回调、`sync_token()` 内存同步方法，与 Go SDK v0.3.6 / TypeScript SDK v0.4.3 功能对齐。
- **`HttpClient::close()` / `Drop`**：停止后台 token 刷新 goroutine，避免长期运行服务中的泄漏。
- **`file_enabled` 标志**：`TokenManager::set_token()` 仅在显式调用 `with_token_file_path()` 后才写文件，防止意外写入默认路径。
- **`Arc<RwLock<ClientConfig>>`**：`HttpClient` 将 config 包裹为共享引用，支持后台任务安全更新 token。
- **`HttpClient::query_token()` / `refresh_token()` / `start_token_auto_refresh()`**：手动刷新与自动刷新控制接口。
- **`src/client/decode.rs`**：提取共用 `decode_value<T>()` 函数，消除 quote/trade 模块重复代码。

### Fixed

- **Push 死锁修正**：`Connected` case 下 `callbacks` 与 `state` 互斥锁获取顺序颠倒导致死锁；重构为提前写状态后再获取 callbacks 锁。
- **`Transaction` 响应模型修正**（对应 Go SDK v0.3.1）：`transacted_at` 类型 `i64` → `String`；新增 `account_id`、`filled_price`、`filled_amount`、`filled_quantity_scale`、`transaction_time` 字段。
- **`FundingHistoryItem` 字段修正**（对应 Go SDK v0.3.4）：`id` 类型 `String` → `i64`，`submit_time`/`update_time` → `created_at`/`updated_at`，移除不存在字段，新增 `ref_id`/`type_`/`type_desc`/`business_date`/`status_desc`/`completed_status`。
- **`SegmentFund.id` 类型修正**（对应 Go SDK v0.3.2）：改为 `serde_json::Value` 兼容服务端可能返回数字或字符串。
- **`FutureKline` 补充 `contract_code` 字段**：服务端实际返回 `contractCode`，原结构体缺失。
- **`get_future_trade_ticks` 响应解包修正**（对应 Go SDK v0.3.3）：服务端返回 `{contractCode, items:[...]}` 包装，使用 `FutureTickWrap` 先解包再回填 `contract_code`。
- **`get_funding_history` 反序列化修正**（对应 Go SDK v0.3.3）：服务端返回裸 list，从 `decode_items` 改为 `decode_value`。
- **重试非幂等写操作**：`TRADE_OPERATIONS` 补充 `place_order`/`modify_order`/`cancel_order`/`place_forex_order` 四个方法，防止误触发重试。
- **`max_retry_time` deadline 实际生效**：原实现未在循环中检查 deadline，修正为每次 retry 前检查。
- **`eprintln!` 替换为 `tracing::info!`**：token 刷新日志改用结构化日志。

## [0.4.0] - 2026-05-09

本次发布达到与 Python / Java / Go / TypeScript SDK **100% API 覆盖**。新增约 65 个方法，重构 12 个方法签名，OrderStatus 枚举对齐 Java SDK。包含多处 breaking change。

### Added

**Trade (17 个新方法)**

- `get_order(req)` — 按 ID 查询单个订单（wire: `orders`，传 id/order_id，返回单个对象）
- `get_managed_accounts(req)` — 查询机构子账户列表（`accounts`）
- `get_derivative_contracts(req)` — 衍生品合约列表（`quote_contract`）
- `get_analytics_asset(req)` — 按日资产分析（`analytics_asset`）
- `get_aggregate_assets(req)` — 综合账户资产汇总（`aggregate_assets`）
- `get_estimate_tradable_quantity(req)` — 可交易数量估算（`estimate_tradable_quantity`）
- `place_forex_order(req)` — 外汇下单（`place_forex_order`）
- `get_segment_fund_available(req)` / `get_segment_fund_history(req)` / `transfer_segment_fund(req)` / `cancel_segment_fund(req)` — 子账户资金调拨
- `get_fund_details(req)` — 资金流水明细（`fund_details`）
- `get_funding_history(req)` — 资金调拨记录（`transfer_fund`）
- `transfer_position(req)` — 内部转股（`position_transfer`）
- `get_position_transfer_records(req)` / `get_position_transfer_detail(req)` / `get_position_transfer_external_records(req)` — 转股记录查询

**Quote (~45 个新方法)**

- 股票基础(15): `get_symbols` / `get_symbol_names` / `get_trade_metas` / `get_stock_details` / `get_stock_delay_briefs` / `get_bars` / `get_bars_by_page` / `get_timeline_history` / `get_trade_rank` / `get_short_interest` / `get_stock_broker` / `get_stock_fundamental` / `get_stock_industry` / `get_quote_permission` / `get_kline_quota`
- 期权扩展(6): `get_option_bars` / `get_option_trade_ticks` / `get_option_timeline` / `get_option_depth` / `get_option_symbols` / `get_option_analysis`
- 期货扩展(10): `get_future_contract` / `get_all_future_contracts` / `get_current_future_contract` / `get_future_continuous_contracts` / `get_future_history_main_contract` / `get_future_bars` / `get_future_bars_by_page` / `get_future_trade_ticks` / `get_future_depth` / `get_future_trading_times`
- 基金(4): `get_fund_symbols` / `get_fund_contracts` / `get_fund_quote` / `get_fund_history_quote`
- 窝轮(2): `get_warrant_briefs` / `get_warrant_filter`
- 行业(2): `get_industry_list` / `get_industry_stocks`
- 公司行动/财务/日历(6): `get_corporate_split` / `get_corporate_dividend` / `get_corporate_earnings_calendar` / `get_financial_currency` / `get_financial_exchange_rate` / `get_trading_calendar`
- 其他(2): `get_market_scanner_tags` / `get_quote_overnight`

**Push (4 对新订阅)**

- `subscribe_cc(symbols)` / `unsubscribe_cc(symbols)` — 加密货币行情（Cc 数据走 `on_quote` 回调）
- `subscribe_market(market)` / `unsubscribe_market(market)` — 市场状态（数据走 `on_quote` 回调）
- StockTop / OptionTop 订阅已在 v0.3.1 中存在，本次确认其 v0.4.0 行为不变

**枚举 (7 个新专属枚举)**

- `OrderSortBy` — 订单排序字段(LATEST_CREATED / LATEST_STATUS_UPDATED)
- `SegmentType` — 账户分部类型(SEC / FUT / FUND / ALL)
- `CorporateActionType` — 公司行动类型(split / dividend / earning)
- `IndustryLevel` — 行业级别(GSECTOR / GGROUP / GIND / GSUBIND)
- `SortDirection` — 排序方向
- `OptionAnalysisPeriod` — 期权分析周期
- `FinancialReportPeriod` — 财报类型(Annual / Quarterly / Ltm)
- `License::Tbms` 新增 `"TBMS"` 变体

**Request structs**

- 新建 `src/model/trade_requests.rs`：`OrdersRequest` / `GetOrderRequest` / `OrderTransactionsRequest` / `PositionsRequest` / `AssetsRequest` 等 18 个 Request struct
- 新建 `src/model/quote_requests.rs`：`BriefRequest` / `TradeTickRequest` / `DepthQuoteRequest` / `FutureBriefRequest` 等 47 个 Request struct

### Changed (BREAKING)

1. **`OrderStatus` 枚举对齐 Java SDK**：删除 `PendingNew` 和 `PartiallyFilled`（Python 客户端派生，服务端不返回）；新增 `PendingSubmit`（code=8）。所有变体添加显式 `#[serde(rename = "...")]`。最终 8 个值：`Invalid(-2)` / `Initial(-1)` / `PendingCancel(3)` / `Cancelled(4)` / `Submitted(5)` / `Filled(6)` / `Inactive(7)` / `PendingSubmit(8)`。新增 `OrderStatus::code()` 方法。

2. **8 个 Trade 方法改签名为 Request struct**：
   - `get_orders()` → `get_orders(req: OrdersRequest)`
   - `get_active_orders()` → `get_active_orders(req: OrdersRequest)`
   - `get_inactive_orders()` → `get_inactive_orders(req: OrdersRequest)`
   - `get_filled_orders(start_ms, end_ms)` → `get_filled_orders(req: OrdersRequest)`（start_ms/end_ms 改为 `req.start_date`/`req.end_date`）
   - `get_order_transactions(id, symbol, sec_type)` → `get_order_transactions(req: OrderTransactionsRequest)`（全字段可选）
   - `get_positions()` → `get_positions(req: PositionsRequest)`
   - `get_assets()` → `get_assets(req: AssetsRequest)`
   - `get_prime_assets()` → `get_prime_assets(req: AssetsRequest)`

3. **4 个 Quote 方法改签名为 Request struct**：
   - `get_brief(symbols: &[&str])` → `get_brief(req: BriefRequest)`
   - `get_trade_tick(symbols: &[&str])` → `get_trade_tick(req: TradeTickRequest)`
   - `get_quote_depth(symbol: &str, market: &str)` → `get_quote_depth(req: DepthQuoteRequest)`
   - `get_future_real_time_quote(contract_codes: &[&str])` → `get_future_real_time_quote(req: FutureBriefRequest)`

### Fixed

- **Push dispatcher Cc dataType bug**：`Cc` 类型的推送数据（加密货币）之前会错误落入 `QuoteBBO` fallback 分支。现已修复，`Cc` 明确路由到 `on_quote` 回调，与 Go/Python/Java SDK 一致。
- **`Order.status` 整数反序列化**：服务端可能返回整数 status 码（如 `6` = Filled），之前 `String` 字段会反序列化失败。现在 `status` 字段使用自定义 deserializer，自动将整数转为 Java 枚举字符串名（`-2→Invalid`, `-1→Initial`, `3→PendingCancel`, `4→Cancelled`, `5→Submitted`, `6→Filled`, `7→Inactive`, `8→PendingSubmit`）。
- **examples 支持 `TIGER_CONFIG_PATH` env var**：不再依赖 CWD 内的配置文件，避免凭证文件被误提交。用法：`TIGER_CONFIG_PATH=~/.tigeropen/tiger_openapi_config.properties cargo run --example trade_example`

### 迁移指引

```rust
// Before (0.3.x)
let orders = tc.get_orders().await?;
let filled = tc.get_filled_orders(start_ms, end_ms).await?;
let txs = tc.get_order_transactions(id, "AAPL", "STK").await?;
let pos = tc.get_positions().await?;
let briefs = qc.get_brief(&["AAPL"]).await?;
let depth = qc.get_quote_depth("AAPL", "US").await?;

// After (0.4.0)
let orders = tc.get_orders(OrdersRequest::default()).await?;
let filled = tc.get_filled_orders(OrdersRequest {
    start_date: Some(start_ms), end_date: Some(end_ms), ..Default::default()
}).await?;
let txs = tc.get_order_transactions(OrderTransactionsRequest {
    order_id: Some(id), symbol: Some("AAPL".into()), sec_type: Some("STK".into()),
    ..Default::default()
}).await?;
let pos = tc.get_positions(PositionsRequest::default()).await?;
let briefs = qc.get_brief(BriefRequest {
    symbols: Some(vec!["AAPL".to_string()]), ..Default::default()
}).await?;
let depth = qc.get_quote_depth(DepthQuoteRequest {
    symbols: Some(vec!["AAPL".to_string()]), market: Some("US".to_string()),
    ..Default::default()
}).await?;

// OrderStatus migration
// OrderStatus::PendingNew   → removed (Python-derived)
// OrderStatus::PartiallyFilled → removed (same)
// OrderStatus::PendingSubmit   → added (maps to server code 8)
```

### 设计原则

- **Request struct 字段名 = 服务端 wire 真名**，Rust 字段天然 snake_case，无需额外 rename。Response struct 用 `#[serde(rename_all = "camelCase")]` 对齐服务端 camelCase 返回。
- 所有 Request 字段 `Option<T>` + `#[serde(skip_serializing_if = "Option::is_none")]`；`account` / `account_id` 留 None 时自动填充 client 初始化的默认账户。

## [0.3.1] - 2026-05-07

### Added

- `OrderStatusData` push message: new fields `updateTime` (field 44, timestamp ms of order info update) and `latestTime` (field 45, timestamp ms of order status update). Generated automatically by `build.rs` via `prost-build`.

## [0.3.0] - 2026-05-06

### Changed (BREAKING)

- **Typed request/response API across all quote and trade methods.** Every
  `QuoteClient` and `TradeClient` method now returns a typed response
  (e.g. `Vec<MarketState>`, `Vec<Brief>`, `Vec<Kline>`, `Vec<Asset>`,
  `Vec<Order>`, `Option<PlaceOrderResult>`) instead of
  `Result<Option<serde_json::Value>, TigerError>`. Callers no longer have
  to destructure `response.data` or `items` envelopes — the client unwraps
  them internally and hands back the domain type.
- **Correct snake_case wire format for requests and camelCase for
  responses.** Per-struct `#[serde(rename_all = "snake_case")]` on
  requests and `#[serde(rename_all = "camelCase")]` on responses gives
  compile-time guarantees that the wire format matches the server
  contract. Previous versions incorrectly sent `"secType"`,
  `"orderType"`, `"totalQuantity"`, etc.
- **Split `Order` into `Order` (response) and `OrderRequest` (request).**
  `Order` exposes the full set of fields returned by the server (all
  camelCase); `OrderRequest` only contains the fields the server accepts
  for `place_order` / `preview_order` / `modify_order` (all snake_case,
  with nested `OrderLegRequest` / `AlgoParamsRequest`). Helper
  factories `market_order` / `limit_order` / `stop_order` /
  `stop_limit_order` / `trail_order` / `auction_limit_order` /
  `auction_market_order` / `algo_order` now return `OrderRequest`.
- **Method signature corrections to match server contract:**
  - `get_brief(symbols)` — uses the `brief` method (not `quote_real_time`).
  - `get_quote_depth(symbol, market)` — `market` is now required.
  - `get_future_contracts(exchange_code)` — method renamed to
    `future_contract_by_exchange_code`; request key is `exchange_code`.
  - `get_future_real_time_quote(contract_codes)` — accepts a slice and
    sends key `contract_codes`.
  - `get_future_kline(req: FutureKlineRequest)` — structured request with
    `contract_codes` / `period` / `begin_time` / `end_time` (both time
    fields default to `-1` when set to `0`).
  - `get_financial_daily(req: FinancialDailyRequest)` /
    `get_financial_report(req: FinancialReportRequest)` /
    `get_corporate_action(req: CorporateActionRequest)` — structured
    request objects for complex parameter sets.
  - `get_capital_flow(symbol, market, period)` and
    `get_capital_distribution(symbol, market)` — flat parameters matching
    the server.
  - `get_option_chain(symbol, expiry)` — `expiry` is a `"YYYY-MM-DD"`
    string that is converted to a UTC millisecond timestamp internally
    and sent in an `option_basic` array (API v3.0).
  - `get_option_brief(identifiers)` — parses OCC identifiers into the
    `option_basic` array with `symbol` / `expiry` (ms) / `right` /
    `strike` (API v2.0).
  - `get_option_kline(identifier, period)` — wraps the parsed identifier
    plus the period in the `option_query` array (API v2.0).
  - `market_scanner(req: MarketScannerRequest)` — structured request.
  - `place_order(order)` returns `Option<PlaceOrderResult>` carrying
    both `id` (internal) and `order_id` (account-level); `modify_order`
    / `cancel_order` return `Option<OrderIdResult>`.
  - `get_order_transactions(order_id, symbol, sec_type)` — sends
    `order_id` as the key; `symbol` / `sec_type` are now required.
  - `get_filled_orders(start_ms, end_ms)` — sends `start_date` /
    `end_date` in milliseconds, both required.
  - `get_quote_contract(symbol, sec_type, expiry)` — wraps the single
    symbol in a `symbols` array and sends `expiry` (e.g. `"20260619"`).
- **Unwrap `{items: [...]}` envelopes for trade endpoints.** `get_orders`,
  `get_active_orders`, `get_inactive_orders`, `get_filled_orders`,
  `get_positions`, `get_assets`, `get_order_transactions`,
  `get_contract(s)`, and `get_quote_contract` now return `Vec<T>`
  directly.
- **`get_corporate_action` flattens the server's `{symbol: [...]}` map**
  into a single `Vec<CorporateAction>`.
- **Removed the old `get_*` compatibility aliases** that delegated to
  methods without the `get_` prefix. The `get_*` names are now the
  canonical signatures.

### Added

- `HttpClient::with_quote_server(config)` — constructor variant that
  wires the HTTP client to `config.quote_server_url` instead of
  `config.server_url`. Use this when constructing a `QuoteClient` so
  quote requests go to the quote gateway.
- `src/model/quote.rs` — 25+ response structs (`MarketState`, `Brief`,
  `Kline` + `KlineItem`, `Timeline` + `TimelineBucket` +
  `TimelineItem`, `TradeTick`, `Depth`, `OptionExpiration`,
  `OptionChain` + `OptionChainRow` + `OptionLeg`, `FutureExchange`,
  `FutureContractInfo`, `FutureQuote`, `FutureKline` +
  `FutureKlineItem`, `FinancialDailyItem`, `FinancialReportItem`,
  `CorporateAction`, `CapitalFlow` + `CapitalFlowItem`,
  `CapitalDistribution`, `ScannerResult` + `ScannerResultItem` +
  `ScannerDataRow`, `QuotePermission`) plus 5 request structs
  (`FinancialDailyRequest`, `FinancialReportRequest`,
  `CorporateActionRequest`, `FutureKlineRequest`,
  `MarketScannerRequest`).
- `src/model/trade.rs` — `Asset`, `AssetSegment`, `PrimeAsset`,
  `PrimeAssetSegment`, `CurrencyAsset`, `PreviewResult`,
  `PlaceOrderResult`, `OrderIdResult`, `Transaction`.
- **Expanded `Order` response fields** to cover what the server actually
  returns (e.g. `external_id`, `filled_cash_amount`, `attr_desc`,
  `attr_list`, `algo_strategy`, `replace_status`, `cancel_status`,
  `can_modify`, `can_cancel`, `is_open`, `trading_session_type`,
  `latest_price`, etc.).
- `examples/quote_example.rs` and `examples/trade_example.rs` — full
  end-to-end coverage of every `QuoteClient` / `TradeClient` method
  with a `PASS / FAIL / SKIP` summary. The trade example runs a real
  low-price limit order (`BUY 1 AAPL @ $1.00`), immediately modifies
  the price, then cancels.
- `VERSION` constant bumped to `"0.3.0"`; HTTP `User-Agent` is
  `openapi-rust-sdk-0.3.0`.

### Fixed

- Double-encoded JSON payloads (the server occasionally returns `data`
  as a JSON string) are transparently parsed by the internal
  `decode_value` helper.

### Unchanged

- Push / streaming client (`src/push`) is protobuf-based and already
  uses the correct wire format — not affected by this release.

## [0.2.0] - 2026-04-30

- Retry policy, protobuf push client, initial trade/quote clients.

## [0.1.0] - 2026-04-01

- Initial release.
