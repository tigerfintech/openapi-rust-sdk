//! Quote 请求结构体（Batch 4）。
//!
//! 所有字段使用 snake_case（与 wire 协议直接对齐），`Option<T>` + `skip_serializing_if` 跳过空值。

use serde::Serialize;
use crate::error::TigerError;

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
pub struct KlineRequest {
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
pub struct KlineByPageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
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

/// 期权链查询条目：symbol + expiry（毫秒时间戳）。
///
/// 构造方式：
/// - `OptionChainItem::new("AAPL", 1705622400000)` — 直接传时间戳
/// - `OptionChainItem::from_date("AAPL", "2024-01-19")?` — 日期字符串，按 symbol 推断时区
/// - `OptionChainItem::from_date_tz("AAPL", "2024-01-19", "America/New_York")?` — 指定时区
#[derive(Debug, Clone, Serialize)]
pub struct OptionChainItem {
    pub symbol: String,
    pub expiry: i64,
}

impl OptionChainItem {
    pub fn new(symbol: impl Into<String>, expiry: i64) -> Self {
        Self { symbol: symbol.into(), expiry }
    }

    pub fn from_date(symbol: impl Into<String>, date: &str) -> Result<Self, TigerError> {
        let symbol = symbol.into();
        let tz = infer_option_timezone(&symbol);
        let expiry = date_to_expiry_ms(date, tz)?;
        Ok(Self { symbol, expiry })
    }

    pub fn from_date_tz(symbol: impl Into<String>, date: &str, timezone: &str) -> Result<Self, TigerError> {
        let symbol = symbol.into();
        let expiry = date_to_expiry_ms(date, timezone)?;
        Ok(Self { symbol, expiry })
    }
}

/// 期权链请求。wire: option_chain (v3.0)
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionChainRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_basic: Option<Vec<OptionChainItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

impl OptionChainRequest {
    pub fn new(items: Vec<OptionChainItem>) -> Self {
        Self { option_basic: Some(items), market: None, lang: None }
    }
}

/// 期权实时行情查询条目：OCC 已解析字段 + expiry 时间戳。
///
/// 构造方式：
/// - `OptionContractItem::from_occ("AAPL 240119C00150000")?` — OCC 格式，按 symbol 推断时区
/// - `OptionContractItem::from_occ_tz("AAPL 240119C00150000", "America/New_York")?` — 指定时区
/// - `OptionContractItem::new("AAPL", 1705622400000, "Call", "150")` — 直接构造
#[derive(Debug, Clone, Serialize)]
pub struct OptionContractItem {
    pub symbol: String,
    pub expiry: i64,
    pub right: String,
    pub strike: String,
}

impl OptionContractItem {
    pub fn new(symbol: impl Into<String>, expiry: i64, right: impl Into<String>, strike: impl Into<String>) -> Self {
        Self { symbol: symbol.into(), expiry, right: right.into(), strike: strike.into() }
    }

    pub fn from_occ(identifier: &str) -> Result<Self, TigerError> {
        let c = parse_occ_identifier(identifier)?;
        let tz = infer_option_timezone(&c.symbol);
        let expiry = date_to_expiry_ms(&c.expiry_date, tz)?;
        Ok(Self { symbol: c.symbol, expiry, right: c.right, strike: c.strike })
    }

    pub fn from_occ_tz(identifier: &str, timezone: &str) -> Result<Self, TigerError> {
        let c = parse_occ_identifier(identifier)?;
        let expiry = date_to_expiry_ms(&c.expiry_date, timezone)?;
        Ok(Self { symbol: c.symbol, expiry, right: c.right, strike: c.strike })
    }
}

/// 期权实时行情请求。wire: option_brief (v2.0)
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionQuoteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_basic: Option<Vec<OptionContractItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

impl OptionQuoteRequest {
    pub fn new(items: Vec<OptionContractItem>) -> Self {
        Self { option_basic: Some(items), market: None, lang: None }
    }
}

/// 期权 K 线查询条目。
#[derive(Debug, Clone, Serialize)]
pub struct OptionKlineItem {
    pub symbol: String,
    pub expiry: i64,
    pub right: String,
    pub strike: String,
    pub period: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

impl OptionKlineItem {
    pub fn new(symbol: impl Into<String>, expiry: i64, right: impl Into<String>, strike: impl Into<String>, period: impl Into<String>) -> Self {
        Self { symbol: symbol.into(), expiry, right: right.into(), strike: strike.into(), period: period.into(), begin_time: None, end_time: None, limit: None }
    }

    pub fn from_occ(identifier: &str, period: impl Into<String>) -> Result<Self, TigerError> {
        let c = parse_occ_identifier(identifier)?;
        let tz = infer_option_timezone(&c.symbol);
        let expiry = date_to_expiry_ms(&c.expiry_date, tz)?;
        Ok(Self { symbol: c.symbol, expiry, right: c.right, strike: c.strike, period: period.into(), begin_time: None, end_time: None, limit: None })
    }

    pub fn from_occ_tz(identifier: &str, period: impl Into<String>, timezone: &str) -> Result<Self, TigerError> {
        let c = parse_occ_identifier(identifier)?;
        let expiry = date_to_expiry_ms(&c.expiry_date, timezone)?;
        Ok(Self { symbol: c.symbol, expiry, right: c.right, strike: c.strike, period: period.into(), begin_time: None, end_time: None, limit: None })
    }
}

// ---- 时区推断 & 日期转时间戳 ------------------------------------------------

/// 按 symbol 推断期权到期日时区（与 Java SDK 对齐）。
pub(crate) fn infer_option_timezone(symbol: &str) -> &'static str {
    // HK options: numeric symbols or known HK suffixes
    if symbol.chars().all(|c| c.is_ascii_digit()) || symbol.ends_with(".HK") {
        return "Asia/Hong_Kong";
    }
    // US options: pure ASCII letters (no dot/digit suffix)
    if symbol.chars().all(|c| c.is_ascii_alphabetic()) {
        return "America/New_York";
    }
    "Asia/Shanghai"
}

/// 日期字符串 "YYYY-MM-DD" → 毫秒时间戳，使用指定时区的午夜 00:00:00。
pub(crate) fn date_to_expiry_ms(date: &str, timezone: &str) -> Result<i64, TigerError> {
    use chrono::NaiveDate;
    use chrono_tz::Tz;

    let tz: Tz = timezone.parse().map_err(|_| {
        TigerError::Config(format!("unknown timezone: {:?}", timezone))
    })?;
    let d = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|e| {
        TigerError::Config(format!("invalid date {:?}: expected YYYY-MM-DD: {}", date, e))
    })?;
    let dt = d.and_hms_opt(0, 0, 0).unwrap();
    let zdt = dt.and_local_timezone(tz).earliest().ok_or_else(|| {
        TigerError::Config(format!("ambiguous or invalid local time for date {:?} in tz {:?}", date, timezone))
    })?;
    Ok(zdt.timestamp_millis())
}

// ---- OCC identifier 解析 ----------------------------------------------------

struct OccContract {
    symbol: String,
    expiry_date: String, // "YYYY-MM-DD"
    right: String,
    strike: String,
}

fn parse_occ_identifier(identifier: &str) -> Result<OccContract, TigerError> {
    let trimmed = identifier.trim();
    // OCC standard pads symbol to 6 chars with spaces, so there may be multiple spaces between
    // symbol and suffix (e.g. "AAPL  260918C00275000"). Split on any whitespace and rejoin.
    let mut parts = trimmed.splitn(2, |c: char| c == ' ');
    let symbol = parts.next().filter(|s| !s.is_empty())
        .ok_or_else(|| TigerError::Config(format!("invalid OCC identifier: {:?}", identifier)))?
        .to_string();
    let rest = parts.next()
        .ok_or_else(|| TigerError::Config(format!("invalid OCC identifier (missing suffix): {:?}", identifier)))?
        .trim_start(); // strip padding spaces between symbol and date
    if rest.len() < 15 {
        return Err(TigerError::Config(format!("invalid OCC identifier (suffix too short): {:?}", identifier)));
    }
    // YYMMDD + C/P + 8-digit strike (5 integer + 3 decimal, no dot)
    let date_part = &rest[..6];
    let right_char = &rest[6..7];
    let strike_raw = &rest[7..];
    let year = 2000 + date_part[..2].parse::<i32>().map_err(|_| TigerError::Config(format!("invalid OCC date: {:?}", identifier)))?;
    let month = date_part[2..4].parse::<u32>().map_err(|_| TigerError::Config(format!("invalid OCC date: {:?}", identifier)))?;
    let day = date_part[4..6].parse::<u32>().map_err(|_| TigerError::Config(format!("invalid OCC date: {:?}", identifier)))?;
    let expiry_date = format!("{:04}-{:02}-{:02}", year, month, day);
    let right = match right_char {
        "C" => "Call".to_string(),
        "P" => "Put".to_string(),
        _ => return Err(TigerError::Config(format!("invalid OCC right {:?}: {:?}", right_char, identifier))),
    };
    let strike_int: u64 = strike_raw.parse().map_err(|_| TigerError::Config(format!("invalid OCC strike: {:?}", identifier)))?;
    let strike = format!("{:.3}", strike_int as f64 / 1000.0);
    Ok(OccContract { symbol, expiry_date, right, strike })
}

/// 期权 K 线请求。wire: option_kline
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionKlineRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_query: Option<Vec<OptionKlineItem>>,
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

/// 港股期权代码列表请求。wire: all_hk_option_symbols
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
pub struct FutureKlineRequest {
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
pub struct FutureKlineByPageRequest {
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
