//! Quote 请求结构体（Batch 4）。
//!
//! 所有字段使用 snake_case（与 wire 协议直接对齐），`Option<T>` + `skip_serializing_if` 跳过空值。

use serde::Serialize;

// ============================================================================
// 股票基础查询
// ============================================================================

/// 股票实时报价请求。wire: quote_real_time
#[derive(Debug, Clone, Serialize, Default)]
pub struct BriefRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_hour_trading: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 全量合约代码查询请求。wire: all_symbols / all_symbol_names
#[derive(Debug, Clone, Serialize, Default)]
pub struct SymbolsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_otc: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 交易元数据请求。wire: quote_stock_trade
#[derive(Debug, Clone, Serialize, Default)]
pub struct TradeMetasRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 股票详情请求。wire: stock_detail
#[derive(Debug, Clone, Serialize, Default)]
pub struct StockDetailsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 延时行情请求。wire: quote_delay
#[derive(Debug, Clone, Serialize, Default)]
pub struct StockDelayBriefsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 盘口深度请求。wire: quote_depth
#[derive(Debug, Clone, Serialize, Default)]
pub struct DepthQuoteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// K 线查询请求。wire: kline
#[derive(Debug, Clone, Serialize, Default)]
pub struct BarsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_fundamental: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// K 线分页包装请求（客户端分页）。
#[derive(Debug, Clone, Serialize, Default)]
pub struct BarsByPageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 历史分时请求。wire: history_timeline
#[derive(Debug, Clone, Serialize, Default)]
pub struct TimelineHistoryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 逐笔成交请求。wire: trade_tick
#[derive(Debug, Clone, Serialize, Default)]
pub struct TradeTickRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 成交榜单请求。wire: trade_rank
#[derive(Debug, Clone, Serialize, Default)]
pub struct TradeRankRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 做空数据请求。wire: quote_shortable_stocks
#[derive(Debug, Clone, Serialize, Default)]
pub struct ShortInterestRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 经纪商持仓请求。wire: stock_broker
#[derive(Debug, Clone, Serialize, Default)]
pub struct StockBrokerRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 股票基本面请求。wire: stock_fundamental
#[derive(Debug, Clone, Serialize, Default)]
pub struct StockFundamentalRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 股票行业归属请求。wire: stock_industry
#[derive(Debug, Clone, Serialize, Default)]
pub struct StockIndustryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 行情权限查询请求。wire: get_quote_permission
#[derive(Debug, Clone, Serialize, Default)]
pub struct QuotePermissionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// K 线配额查询请求。wire: kline_quota
#[derive(Debug, Clone, Serialize, Default)]
pub struct KlineQuotaRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_details: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

// ============================================================================
// 期权扩展
// ============================================================================

/// 期权查询嵌套条目（option_query / option_basic 用）。
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionQueryItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

/// 期权 K 线请求。wire: option_kline
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionBarsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_query: Option<Vec<OptionQueryItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 期权逐笔请求。wire: option_trade_tick
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionTradeTicksRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts: Option<Vec<OptionQueryItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 期权分时请求。wire: option_timeline
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionTimelineRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_query: Option<Vec<OptionQueryItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 期权盘口请求。wire: option_depth
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionDepthRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_basic: Option<Vec<OptionQueryItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 期权代码列表请求。wire: option_symbol
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionSymbolsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 期权分析请求。wire: option_analysis
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionAnalysisRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_volatility_list: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

// ============================================================================
// 期货扩展
// ============================================================================

/// 期货实时行情请求。wire: future_real_time_quote
#[derive(Debug, Clone, Serialize, Default)]
pub struct FutureBriefRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_codes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 按 contract_code 查询期货合约请求。wire: future_contract_by_contract_code / future_current_contract
#[derive(Debug, Clone, Serialize, Default)]
pub struct FutureContractSingleRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_code: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub contract_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 查询所有期货合约请求。wire: future_contracts
#[derive(Debug, Clone, Serialize, Default)]
pub struct AllFutureContractsRequest {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub contract_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 连续主力合约请求。wire: future_continuous_contracts
#[derive(Debug, Clone, Serialize, Default)]
pub struct FutureContinuousContractsRequest {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub contract_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 主力合约历史请求。wire: future_main_contract
#[derive(Debug, Clone, Serialize, Default)]
pub struct FutureHistoryMainContractRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_codes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 期货 K 线请求（含索引分页）。wire: future_kline
#[derive(Debug, Clone, Serialize, Default)]
pub struct FutureBarsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_codes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 期货 K 线分页包装请求（客户端分页）。
#[derive(Debug, Clone, Serialize, Default)]
pub struct FutureBarsByPageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 期货逐笔请求。wire: future_tick (API version 3.0)
#[derive(Debug, Clone, Serialize, Default)]
pub struct FutureTradeTicksRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 期货盘口请求。wire: future_depth
#[derive(Debug, Clone, Serialize, Default)]
pub struct FutureDepthRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_codes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 期货交易时段请求。wire: future_trading_date
#[derive(Debug, Clone, Serialize, Default)]
pub struct FutureTradingTimesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trading_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

// ============================================================================
// 基金 / 窝轮 / 行业 / 财务 / 日历 / 其他
// ============================================================================

/// 基金代码列表请求。wire: fund_all_symbols
#[derive(Debug, Clone, Serialize, Default)]
pub struct FundSymbolsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 基金合约请求。wire: fund_contracts
#[derive(Debug, Clone, Serialize, Default)]
pub struct FundContractsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 基金实时净值请求。wire: fund_quote
#[derive(Debug, Clone, Serialize, Default)]
pub struct FundQuoteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 基金历史净值请求。wire: fund_history_quote
#[derive(Debug, Clone, Serialize, Default)]
pub struct FundHistoryQuoteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 窝轮简要报价请求。wire: warrant_briefs
#[derive(Debug, Clone, Serialize, Default)]
pub struct WarrantBriefsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 窝轮筛选请求。wire: warrant_filter
#[derive(Debug, Clone, Serialize, Default)]
pub struct WarrantFilterRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_field_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_ym: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 行业列表请求。wire: industry_list
#[derive(Debug, Clone, Serialize, Default)]
pub struct IndustryListRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 行业下股票列表请求。wire: industry_stock_list
#[derive(Debug, Clone, Serialize, Default)]
pub struct IndustryStocksRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 财报币种请求。wire: financial_currency
#[derive(Debug, Clone, Serialize, Default)]
pub struct FinancialCurrencyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 汇率请求。wire: financial_exchange_rate
#[derive(Debug, Clone, Serialize, Default)]
pub struct FinancialExchangeRateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 交易日历请求。wire: trading_calendar
#[derive(Debug, Clone, Serialize, Default)]
pub struct TradingCalendarRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 扫描器标签请求。wire: market_scanner_tags
/// 注意 wire 字段名是 `multi_tags_fields`（不是 `multi_tag_field_list`）。
#[derive(Debug, Clone, Serialize, Default)]
pub struct MarketScannerTagsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_tags_fields: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 隔夜行情请求。wire: quote_overnight
#[derive(Debug, Clone, Serialize, Default)]
pub struct QuoteOvernightRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}
