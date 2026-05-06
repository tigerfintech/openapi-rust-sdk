# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
