# Changelog

## 0.7.0 (2026-11-18)

### New

- Contract responses expose supported combinations of order types, time in force, and trading sessions through order capabilities.
- `get_contract_v3` queries a single trading contract, including option order capabilities and account-specific trading properties.

### Breaking

- `Contract` struct literals must include `order_capabilities`; use `None` when capabilities are unavailable. Existing contract constructors remain compatible.

## 0.6.0 (2026-11-16)

### New

- US option snapshots and option chains provide GTH/CURB extended-session quotes and independent cumulative volume and turnover.
- Protobuf option callbacks provide current-session prices, volume, and turnover with GTH/CURB session tags and independent cumulative totals.
- Option minute-bar requests support `with_hour_trading` to include or exclude extended-session data.

### Breaking

- `OptionKlineItem` struct literals must include `with_hour_trading`; set it to `None` to retain the server default, or use the existing constructors.
- Exhaustive option-quote and Protobuf-message struct literals must include the new fields; use `..Default::default()` where appropriate.

## 0.5.13 (2026-11-11)

### New

- `NewsClient.get_stock_news` queries stock news with page-number pagination.
- `NewsClient.get_market_news` queries market highlights with cursor pagination.

## 0.5.12 (2026-11-10)

### New

- Asset and aggregate-asset queries support `asset_quote_type` values `ETH`, `RTH`, and `OVERNIGHT`.
- Asset and position subscriptions support quote-basis selection through new subscription and message-building methods.
- Account subscriptions retain their quote basis after reconnecting.

### Changed

- Rust 1.88 or later is required.
- Existing subscription methods retain their signatures and behavior.

## 0.5.11 (2026-08-27)

### New

- Real-time stock and cryptocurrency quotes provide turnover through `Brief::amount`.

## 0.5.10 (2026-08-19)

### New

- `PushClientOptions.use_full_tick` enables full-tick push messages; existing constructors retain their default behavior.
- Option quotes provide mark prices and midpoint prices.
- Trade ticks provide readable trade-condition descriptions.
- K-line and timeline responses provide optional fractional volume.
- `get_timeline_with_request` accepts `sec_type` for cryptocurrency queries, and paginated K-line queries retain the security type.

### Fixed

- Overnight quote models match the service response, including prices, bid/ask data, timestamps, trading status, and amplitude.

### Breaking

- `Callbacks::on_tick` receives `PushTradeTick` instead of `pb::TradeTickData`. Update callback signatures and access decoded values such as `.ticks[i].price`.
- `ForexOrderResult.id` is now `i64` instead of `String`.
- Fund-detail date filters require `Option<String>` with `yyyy-MM-dd` values instead of millisecond timestamps.
- Move `AlgoParams.algo_strategy` assignments to the top-level `OrderRequest.algo_strategy` field.
- Overnight quote models no longer include unsupported open/high/low/close or start/end-time fields.

## 0.5.9 (2026-08-04)

### New

- Quote, trading, and order response models support JSON serialization for local storage.

## 0.5.8 (2026-07-24)

### New

- Quote and trading clients expose token-query, token-refresh, and automatic-refresh controls.
- Properties files support custom trading URLs, quote URLs, and service public keys.
- A service-public-key constant is available for custom configuration.
- Order helpers support monetary market and limit orders, trailing-stop prices, multi-leg orders, combination orders, and OCA orders.

### Changed

- `iceberg_order` combines basic and extended options in one function.
- Trading requests consistently inherit configured institutional secret keys.

### Fixed

- Subscription acknowledgements without quote payloads no longer trigger error callbacks.
- Token files are discovered next to configuration files independently of the working directory.
- Token queries decode double-encoded JSON responses.
- Explicit secret-key values are detected correctly.

## 0.5.7 (2026-07-23)

### New

- `get_corporate_symbol_change` queries stock symbol changes.
- `get_corporate_delisting` queries delisting events.
- `get_corporate_ipo` queries IPO events.
- `CorporateActionType` includes `SymbolChange`, `Delisting`, and `Ipo`.

## 0.5.6 (2026-07-22)

### Fixed

- Response-decoding failures return `TigerError::Parse` instead of configuration errors.
- Properties-file continuations ignore comment lines and distinguish escaped backslashes from continuation markers.

### Breaking

- Exhaustive `TigerError` matches must add a `TigerError::Parse(_)` branch.

## 0.5.5 (2026-07-13)

### New

- Option-chain filters support moneyness, implied volatility, open interest, and Greeks; option analysis supports per-symbol periods.
- Order requests support brackets, multi-leg combinations, GTD expiry times, and institutional allocations.
- Contract responses provide the primary exchange.

### Fixed

- Single-order queries use the correct endpoint.
- Paginated K-line queries retain results for every requested symbol.
- Large retry counts no longer cause backoff calculation to panic.
- Token-refresh timestamps avoid truncation beyond the 32-bit epoch range.
- Fund-detail IDs decode service-provided numeric strings.
- Market-scanner tag queries send the correct multi-tag field name.

### Breaking

- Option-analysis fields now match the service response: replace `implied_volatility` with `implied_vol30_days`, `historical_volatility30_day` with `his_volatility`, `historical_volatility60_day` with `iv_his_v_ratio`, and `historical_volatility90_day` with `call_put_ratio`. The model also provides `implied_vol_metric`.
- `OptionVolatilityPoint.date` and `volatility` are replaced by `implied_vol`, `percentile`, `rank`, `his_volatility`, and `timestamp`.
- `FundDetails.id` is now `String` instead of `i64`.

## 0.5.3 (2026-07-08)

### Changed

- `get_brief`, `get_option_brief`, `get_stock_delay_briefs`, and `get_warrant_briefs` are deprecated. Use `get_real_time_quote`, `get_option_quote`, `get_delayed_quote`, and `get_warrant_quote`, respectively.

## 0.5.0 (2026-07-07)

### New

- `QuoteClient::from_config` and `TradeClient::from_config` construct clients directly from configuration.
- Generic `call_*` methods are public for custom requests.

### Breaking

- Existing `new` and `with_secret_key` constructors now take ownership of `HttpClient` instead of borrowing it.
- Quote methods accept multiple symbols: K-line and option-expiration calls take symbol slices, option chains take symbol/expiry pairs, and option K-lines take identifier slices.
- `get_kline` requires `KlineRequest`; replace `get_bars` and `BarsRequest` with `get_kline` and `KlineRequest`.
- Replace `get_bars_by_page` and `BarsByPageRequest` with `get_kline_by_page` and `KlineByPageRequest`, using a symbol vector instead of one symbol.
- `get_option_bars` is removed; use `get_option_kline`.
- Replace `get_future_bars` and `FutureBarsRequest` with `get_future_kline` and `FutureKlineRequest`.
- Replace `get_future_bars_by_page` and `FutureBarsByPageRequest` with `get_future_kline_by_page` and `FutureKlineByPageRequest`.

## 0.4.4 (2026-07-03)

### Fixed

- Trading examples use configured institutional secret keys.

## 0.4.3 (2026-06-24)

### New

- `iceberg_order` creates iceberg orders with optional display size, check intervals, price type, and execution times.
- Order models expose iceberg-order fields.
- `TradeClient::preview_order` previews an order.

### Changed

- `get_brief`, `get_option_brief`, `get_stock_delay_briefs`, and `get_warrant_briefs` are deprecated in favor of `get_real_time_quote`, `get_option_quote`, `get_delayed_quote`, and `get_warrant_quote`.

## 0.4.2 (2026-06-09)

### New

- Option-exercise APIs support checks, eligible positions, submission, records, and cancellation.
- `TradeClient::with_secret_key` configures institutional authentication.
- Secret keys can be loaded from properties files or set through the configuration builder.

### Changed

- Non-idempotent exercise submission and cancellation are excluded from automatic retries.

### Fixed

- Response decoding preserves the original error.

## 0.4.1 (2026-05-25)

### New

- Automatic token refresh supports custom loader/writer callbacks and in-memory token synchronization.
- `HttpClient::close` and client destruction stop background refresh.
- Token files are written only when a token-file path is explicitly configured.
- Configuration updates are shared safely with background token-refresh tasks.
- HTTP clients expose token-query, refresh, and automatic-refresh controls.

### Fixed

- Push connections no longer deadlock during connection callbacks.
- Transaction timestamps decode as strings and additional execution fields are available.
- Funding-history fields match the service response, including numeric IDs, creation/update times, references, descriptions, and completion status.
- Segment-fund IDs accept numbers or strings.
- Futures K-lines expose their contract code, and tick responses retain it after decoding.
- Funding-history responses decode bare lists.
- Order placement, modification, cancellation, and forex placement are excluded from automatic retries.
- Retry deadlines are enforced.

## 0.4.0 (2026-05-09)

### New

- Trading APIs add individual orders, managed accounts, derivative contracts, asset analytics and aggregation, tradable quantities, forex orders, funding, and position transfers.
- Quote APIs add stock data, extended option and futures queries, funds, warrants, industries, corporate actions, financials, and calendars.
- Cryptocurrency and market-status push subscriptions deliver data through `on_quote`.
- Added supporting enums and the `License::Tbms` variant.
- `OrderStatus::code` exposes numeric status codes.

### Fixed

- Cryptocurrency push messages reach the correct callback.
- Numeric order statuses decode to their enum values.
- Examples accept configuration paths through `TIGER_CONFIG_PATH`.

### Breaking

- Replace `OrderStatus::PendingNew` and `OrderStatus::PartiallyFilled` with `OrderStatus::PendingSubmit`.
- Trading and quote queries require request structs instead of positional arguments. Update calls as shown below.

```rust
// Before
let orders = tc.get_orders().await?;
let filled = tc.get_filled_orders(start_ms, end_ms).await?;
let txs = tc.get_order_transactions(id, "AAPL", "STK").await?;
let pos = tc.get_positions().await?;
let briefs = qc.get_brief(&["AAPL"]).await?;
let depth = qc.get_quote_depth("AAPL", "US").await?;

// After
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
```

## 0.3.1 (2026-05-07)

### New

- Order-status push messages provide order-information and order-status update timestamps.

## 0.3.0 (2026-05-06)

### New

- Quote and trading methods provide typed response models, with expanded order information.
- `HttpClient::with_quote_server` constructs a client for the configured quote service.
- Quote and trading examples cover client operations.

### Fixed

- Double-encoded JSON responses parse correctly.
- SDK version reporting matches the release.

### Breaking

- Quote and trading methods return typed results instead of raw JSON responses. Collection wrappers are removed, and corporate-action maps become flat vectors.
- Request fields use snake_case, and response fields use camelCase JSON names.
- Use `OrderRequest` for placement, preview, and modification; `Order` remains the response type. Order helpers now return `OrderRequest`.
- Quote-depth queries require a market. Futures contracts require an exchange code, and futures quotes accept a slice of contract codes.
- Futures K-lines, financials, corporate actions, and market scanning require their corresponding request structs.
- Capital-flow queries take symbol, market, and period; capital-distribution queries take symbol and market.
- Option chains require an expiry date string; option quotes and K-lines accept option identifiers.
- Placement returns `Option<PlaceOrderResult>` with internal and account-level order IDs. Modification and cancellation return `Option<OrderIdResult>`.
- Order-transaction queries require order ID, symbol, and security type. Filled-order queries require begin and end timestamps in milliseconds.
- Quote-contract queries require symbol, security type, and expiry.
- The `get_*` method names are canonical; earlier compatibility aliases are removed.

## 0.2.0 (2026-04-30)

### New

- Added retries, Protobuf push connections, and initial trading and quote clients.

## 0.1.0 (2026-04-01)

### New

- Initial release.
