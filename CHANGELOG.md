# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.10] - 2026-08-19

### Breaking

- `Callbacks::on_tick` 回调参数类型由原始 `pb::TradeTickData` 改为解码后的 `PushTradeTick`，每笔成交以 `PushTick` 结构体返回，包含 `price`、`volume`、`cond`、`part_code`、`part_name` 等字段。迁移方式：将注册回调的闭包入参类型从 `pb::TradeTickData` 改为 `PushTradeTick`，原字段通过 `.ticks[i].price` 等访问。
- `ForexOrderResult.id` 字段类型由 `String` 改为 `i64`，与服务端 wire 类型对齐。原按 `String` 访问此字段的代码需改为 `i64`。
- `FundDetailsRequest.start_date` / `end_date` 字段类型由 `Option<i64>`（毫秒时间戳）改为 `Option<String>`（`yyyy-MM-dd` 格式），与服务端 wire 格式对齐。原传毫秒时间戳的调用方需改为传日期字符串，如 `Some("2024-01-01".to_string())`。
- `AlgoParams.algo_strategy` 字段已从 `AlgoParams` 移至 `OrderRequest` 顶层（`OrderRequest.algo_strategy`）。原通过 `AlgoParams.algo_strategy` 赋值的代码需迁移至 `OrderRequest.algo_strategy`，否则该值会被静默丢弃。

### Added

- `PushClientOptions.use_full_tick: Option<bool>` 新增字段，设为 `Some(true)` 时在连接认证时请求完整逐笔成交推送；现有 `PushClient::new` 默认 `false` 保持向后兼容
- `OptionLeg` 新增 `mark_price`、`pre_mark_price`、`mark_timestamp`、`mid_price`、`pre_mid_price`、`mid_timestamp` 字段
- `TradeTickItem` 新增 `cond` 字段，原始单字符代码已转换为可读字符串（如 `US_REGULAR_SALE`、`HK_AUTOMATCH_NORMAL`）
- `QuoteOvernight` 响应模型与服务端字段对齐，新增最新价、买卖盘、时间戳、交易状态和振幅等字段，并移除服务端不存在的开高低收和起止时间字段
- `KlineItem::volume_decimal` 和 `TimelineItem::volume_decimal` 新增可空字段，支持数字货币小数成交量
- `QuoteClient::get_timeline_with_request` 接受包含 `sec_type` 的 `TimelineRequest`（数字货币使用 `CC`），分页 K 线请求会在每一页保留 `KlineByPageRequest::sec_type`

## [0.5.9] - 2026-08-04

### Added

- 行情、交易、订单响应模型统一派生 `Serialize`，调用者可直接将返回数据序列化为 JSON 用于本地缓存。

## [0.5.8] - 2026-07-24

### Added
- `TradeClient` / `QuoteClient`：新增 `query_token()` / `refresh_token()` / `start_token_auto_refresh()` 方法
- `SANDBOX_TIGER_PUBLIC_KEY` 常量，支持非生产环境公钥配置
- properties 文件支持 `server_url`、`quote_server_url`、`tiger_public_key` 字段
- 订单工具函数：新增 `market_order_by_amount`、`limit_order_by_amount`、`trail_order_by_price`、`limit_order_with_legs`、`combo_order`、`oca_order`、`contract_leg`
- `iceberg_order` 合并可选参数（原基础版与全参版统一为一个函数）

### Fixed

- 推送订阅确认消息不含行情 payload 时不再误触发 `on_error`
- token 文件与 config 文件同目录自动加载，不再依赖当前工作目录
- `query_token` / `query_token_from_config` 正确处理服务端 `data` 双重编码的 JSON 响应
- `inject_secret_key_json`：正确判断 `secret_key` 是否已设置（类型感知）

### Changed
- `TradeClient` / `QuoteClient` 所有交易方法统一通过 `inject_secret_key_json` 注入 `secret_key`

## [0.5.7] - 2026-07-23

### Added
- `get_corporate_symbol_change` — 股票代码变更查询，返回 `Vec<CorporateSymbolChange>`
- `get_corporate_delisting` — 退市事件查询，返回 `Vec<CorporateDelisting>`
- `get_corporate_ipo` — 新股上市查询，返回 `Vec<CorporateIPO>`
- `CorporateActionType`：新增 `SymbolChange`、`Delisting`、`Ipo`

## [0.5.6] - 2026-07-22
### Fixed
- 修复 `parse_api_response` 反序列化失败时返回 `TigerError::Config` 语义错误的问题，改为新增的 `TigerError::Parse` 变体。**注意**：若代码中对 `TigerError` 使用穷举 `match`（无 `_ =>`），需补充 `TigerError::Parse(_)` 分支。
- 修复 properties 配置文件续行（`\`）解析：注释行不再被拼入值，`\\` 不再触发续行。

## [0.5.5] - 2026-07-13

### Bug Fixes

- `TradeClient::get_order` now correctly calls the `order_no` wire method instead of `orders` (which is the batch list API); previously single-order lookup was silently broken.
- `QuoteClient::get_kline_by_page` now correctly accumulates K-line items for **all** symbols in the request. Previously only the first symbol's bars were collected; remaining symbols' data was silently dropped.
- `RetryPolicy::calculate_backoff` no longer panics on large retry counts.
- `TokenManager::should_token_refresh` (both struct method and standalone function) now uses `i64::try_from` for the Unix epoch seconds conversion instead of `as i64` truncation, avoiding the Y2038 correctness hazard.

### Added

- 期权链请求新增筛选条件（ITM、IV/持仓量区间、希腊值区间），期权分析支持按 symbol 单独指定统计周期。
- `OrderRequest` 新增一批字段，支持止盈止损单、MLEG 组合单、GTD 到期时间和机构分仓下单。
- `Contract` 新增 `primary_exchange` 字段，之前访问主交易所返回为空。

### Breaking Changes

- **`OptionAnalysis` 响应模型字段重命名**：原字段名与服务端不符，已全部替换为新字段名。迁移方式：`implied_volatility` 改用 `implied_vol30_days`；`historical_volatility30_day` 改用 `his_volatility`；`historical_volatility60_day` 改用 `iv_his_v_ratio`；`historical_volatility90_day` 改用 `call_put_ratio`；新增 `implied_vol_metric` 字段。
- **`OptionVolatilityPoint` 响应模型字段重命名**：原 `date`/`volatility` 字段已替换为 `implied_vol`、`percentile`、`rank`、`his_volatility`、`timestamp`。

### Fixed

- **`get_order` wire method 错误**：错误的 `"order_no"` → `"orders"`（`"order_no"` 返回的是下单结果结构 `{id, orderId}`，不是 `Order` 对象；与 Go/Python SDK 对齐）。
- **`FundDetails.id` 类型错误**：服务端返回 `id` 为数字字符串（如 `"4733519770"`），原来 `i64` 反序列化直接失败；改为 `String`。
- **`MarketScannerTagsRequest.multi_tags_fields` rename 错误**：服务端 wire 字段名为 `multi_tag_field_list`，而非 `multi_tags_fields`；添加 `#[serde(rename = "multi_tag_field_list")]`。

## [0.5.3] - 2026-07-08

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

## [0.4.0] - 2026-05-09

本次发布达到与 Python / Java / Go / TypeScript SDK 的 API 覆盖对齐，包含多处 breaking change。

### Added

- **交易接口扩展**：新增单个订单查询、机构子账户列表、衍生品合约列表、按日资产分析、综合账户资产汇总、可交易数量估算、外汇下单、子账户资金调拨、资金流水与调拨记录查询、内部转股及转股记录查询。
- **行情接口大幅扩展**：新增股票基础信息、期权扩展查询、期货扩展查询、基金、窝轮、行业分类、公司行动/财务/日历等接口。
- **推送新增两类订阅**：加密货币行情（`subscribe_cc`/`unsubscribe_cc`）和市场状态（`subscribe_market`/`unsubscribe_market`），均通过 `on_quote` 回调返回数据。
- **新增多个业务枚举**：订单排序、账户分部类型、公司行动类型、行业级别、排序方向、期权分析周期、财报类型等，`License` 新增 `Tbms` 变体。
- **交易与行情方法统一改用 Request struct 传参**，替代此前分散的位置参数。

### Changed (BREAKING)

- **`OrderStatus` 枚举对齐服务端语义**：删除服务端不会返回的 `PendingNew`、`PartiallyFilled`；新增 `PendingSubmit`。新增 `OrderStatus::code()` 方法用于获取对应数值码。
- **多个 Trade / Quote 方法改为接受 Request struct 参数**，替代原先的位置参数列表（如 `get_orders`、`get_active_orders`、`get_inactive_orders`、`get_filled_orders`、`get_order_transactions`、`get_positions`、`get_assets`、`get_prime_assets`、`get_brief`、`get_trade_tick`、`get_quote_depth`、`get_future_real_time_quote`）；请参考下方迁移指引更新调用方式。

### Fixed

- **Push dispatcher Cc dataType bug**：加密货币推送数据之前会错误落入 `QuoteBBO` fallback 分支，现已修复为路由到 `on_quote` 回调。
- **`Order.status` 整数反序列化**：服务端返回整数 status 码时原先会反序列化失败，现在自动转换为对应的枚举值。
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
