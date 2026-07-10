//! 行情响应/请求模型。
//!
//! - 响应结构体使用 `#[serde(rename_all = "camelCase")]` 匹配服务端返回。
//! - 请求结构体使用 `#[serde(rename_all = "snake_case")]` 匹配服务端约定。

use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    let v = Value::deserialize(deserializer)?;
    match v {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        Value::Null => Ok(String::new()),
        other => Ok(other.to_string()),
    }
}

// ========== 响应模型 ==========

/// 市场状态
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MarketState {
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub market_status: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub open_time: String,
}

/// 实时快照（quote_real_time）
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Brief {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub open: f64,
    #[serde(default)]
    pub high: f64,
    #[serde(default)]
    pub low: f64,
    #[serde(default)]
    pub close: f64,
    #[serde(default)]
    pub pre_close: f64,
    #[serde(default)]
    pub latest_price: f64,
    #[serde(default)]
    pub latest_time: i64,
    #[serde(default)]
    pub ask_price: f64,
    #[serde(default)]
    pub ask_size: i64,
    #[serde(default)]
    pub bid_price: f64,
    #[serde(default)]
    pub bid_size: i64,
    #[serde(default)]
    pub volume: i64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub adj_pre_close: f64,
    #[serde(default)]
    pub change: f64,
    #[serde(default)]
    pub change_rate: f64,
    #[serde(default)]
    pub amplitude: f64,
    #[serde(default, deserialize_with = "deserialize_string_or_number")]
    pub expiry: String,
    #[serde(default)]
    pub strike: String,
    #[serde(default)]
    pub right: String,
    #[serde(default)]
    pub multiplier: i32,
    #[serde(default)]
    pub open_interest: i64,
}

/// K 线单根
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KlineItem {
    #[serde(default)]
    pub time: i64,
    #[serde(default)]
    pub volume: i64,
    #[serde(default)]
    pub open: f64,
    #[serde(default)]
    pub close: f64,
    #[serde(default)]
    pub high: f64,
    #[serde(default)]
    pub low: f64,
    #[serde(default)]
    pub amount: f64,
}

/// K 线（一个标的一组）
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Kline {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub period: String,
    #[serde(default)]
    pub next_page_token: String,
    #[serde(default)]
    pub items: Vec<KlineItem>,
}

/// 分时点
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TimelineItem {
    #[serde(default)]
    pub time: i64,
    #[serde(default)]
    pub volume: i64,
    #[serde(default)]
    pub price: f64,
    #[serde(default)]
    pub avg_price: f64,
}

/// 分时数据块
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TimelineBucket {
    #[serde(default)]
    pub items: Vec<TimelineItem>,
}

/// 分时
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub period: String,
    #[serde(default)]
    pub pre_close: f64,
    #[serde(default)]
    pub intraday: Option<TimelineBucket>,
    #[serde(default)]
    pub pre_hours: Option<TimelineBucket>,
    #[serde(default)]
    pub after_hours: Option<TimelineBucket>,
}

/// 逐笔成交
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TradeTickItem {
    #[serde(default)]
    pub time: i64,
    #[serde(default)]
    pub volume: i64,
    #[serde(default)]
    pub price: f64,
    #[serde(default)]
    pub r#type: String,
}

/// 逐笔（一个标的一组）
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TradeTick {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub begin_index: i64,
    #[serde(default)]
    pub end_index: i64,
    #[serde(default)]
    pub items: Vec<TradeTickItem>,
}

/// 深度一档
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DepthLevel {
    #[serde(default)]
    pub price: f64,
    #[serde(default)]
    pub count: i32,
    #[serde(default)]
    pub volume: i64,
}

/// 深度报价
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Depth {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub asks: Vec<DepthLevel>,
    #[serde(default)]
    pub bids: Vec<DepthLevel>,
}

/// 期权到期日
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptionExpiration {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub option_symbols: Vec<String>,
    #[serde(default)]
    pub dates: Vec<String>,
    #[serde(default)]
    pub timestamps: Vec<i64>,
    #[serde(default)]
    pub periods: Vec<String>,
    #[serde(default)]
    pub counts: Vec<i32>,
}

/// 单腿期权数据
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptionLeg {
    #[serde(default)]
    pub identifier: String,
    #[serde(default)]
    pub strike: String,
    #[serde(default)]
    pub right: String,
    #[serde(default)]
    pub bid_price: f64,
    #[serde(default)]
    pub bid_size: i64,
    #[serde(default)]
    pub ask_price: f64,
    #[serde(default)]
    pub ask_size: i64,
    #[serde(default)]
    pub volume: i64,
    #[serde(default)]
    pub latest_price: f64,
    #[serde(default)]
    pub pre_close: f64,
    #[serde(default)]
    pub open_interest: i64,
    #[serde(default)]
    pub multiplier: i32,
    #[serde(default)]
    pub last_timestamp: i64,
    #[serde(default)]
    pub implied_vol: f64,
    #[serde(default)]
    pub delta: f64,
    #[serde(default)]
    pub gamma: f64,
    #[serde(default)]
    pub theta: f64,
    #[serde(default)]
    pub vega: f64,
    #[serde(default)]
    pub rho: f64,
}

/// Put/Call 配对
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptionChainRow {
    #[serde(default)]
    pub put: Option<OptionLeg>,
    #[serde(default)]
    pub call: Option<OptionLeg>,
}

/// 期权链
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptionChain {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub expiry: i64,
    #[serde(default)]
    pub items: Vec<OptionChainRow>,
}

/// 期权快照（与 Brief 类似，单独类型方便未来扩展）
pub type OptionBrief = Brief;

/// 期权 K 线
pub type OptionKline = Kline;

/// 期货交易所
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureExchange {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub zone_id: String,
}

/// 期货合约详情
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureContractInfo {
    #[serde(default)]
    pub continuous: bool,
    #[serde(default)]
    pub trade: bool,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub contract_code: String,
    #[serde(default)]
    pub ib_code: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub contract_month: String,
    #[serde(default)]
    pub last_trading_date: String,
    #[serde(default)]
    pub first_notice_date: String,
    #[serde(default)]
    pub last_bidding_close_time: i64,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub exchange_code: String,
    #[serde(default)]
    pub multiplier: f64,
    #[serde(default)]
    pub min_tick: f64,
    #[serde(default)]
    pub display_multiplier: f64,
    #[serde(default)]
    pub exchange: String,
    #[serde(default)]
    pub product_worth: String,
    #[serde(default)]
    pub delivery_mode: String,
    #[serde(default)]
    pub product_type: String,
    #[serde(default)]
    pub product_scale: String,
    #[serde(default)]
    pub last_trading_timestamp: i64,
}

/// 期货实时报价
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureQuote {
    #[serde(default)]
    pub contract_code: String,
    #[serde(default)]
    pub latest_price: f64,
    #[serde(default)]
    pub latest_size: i64,
    #[serde(default)]
    pub latest_time: i64,
    #[serde(default)]
    pub bid_price: f64,
    #[serde(default)]
    pub ask_price: f64,
    #[serde(default)]
    pub bid_size: i64,
    #[serde(default)]
    pub ask_size: i64,
    #[serde(default)]
    pub open_interest: i64,
    #[serde(default)]
    pub open_interest_change: i64,
    #[serde(default)]
    pub volume: i64,
    #[serde(default)]
    pub open: f64,
    #[serde(default)]
    pub high: f64,
    #[serde(default)]
    pub low: f64,
    #[serde(default)]
    pub settlement: f64,
    #[serde(default)]
    pub limit_up: f64,
    #[serde(default)]
    pub limit_down: f64,
    #[serde(default)]
    pub avg_price: f64,
}

/// 期货 K 线单根
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureKlineItem {
    #[serde(default)]
    pub time: i64,
    #[serde(default)]
    pub volume: i64,
    #[serde(default)]
    pub open: f64,
    #[serde(default)]
    pub close: f64,
    #[serde(default)]
    pub high: f64,
    #[serde(default)]
    pub low: f64,
    #[serde(default)]
    pub last_time: i64,
    #[serde(default)]
    pub open_interest: i64,
    #[serde(default)]
    pub settlement: f64,
}

/// 期货 K 线（一个合约一组）
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureKline {
    /// 合约代码（服务端字段名 contractCode）
    #[serde(default)]
    pub contract_code: String,
    #[serde(default)]
    pub next_page_token: String,
    #[serde(default)]
    pub items: Vec<FutureKlineItem>,
}

/// 日级财务数据
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FinancialDailyItem {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub field: String,
    #[serde(default)]
    pub date: i64,
    #[serde(default)]
    pub value: f64,
}

/// 财报数据
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportItem {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub field: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub filing_date: String,
    #[serde(default)]
    pub period_end_date: String,
}

/// 公司行动
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CorporateAction {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub exchange: String,
    #[serde(default)]
    pub execute_date: String,
    #[serde(default)]
    pub action_type: String,
    #[serde(default)]
    pub record_date: String,
    #[serde(default)]
    pub announced_date: String,
    #[serde(default)]
    pub pay_date: String,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub from_factor: f64,
    #[serde(default)]
    pub to_factor: f64,
}

/// 资金流向明细
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CapitalFlowItem {
    #[serde(default)]
    pub time: String,
    #[serde(default)]
    pub timestamp: i64,
    #[serde(default)]
    pub net_inflow: f64,
}

/// 资金流向
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CapitalFlow {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub period: String,
    #[serde(default)]
    pub items: Vec<CapitalFlowItem>,
}

/// 资金分布
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CapitalDistribution {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub net_inflow: f64,
    #[serde(default)]
    pub in_all: f64,
    #[serde(default)]
    pub in_big: f64,
    #[serde(default)]
    pub in_mid: f64,
    #[serde(default)]
    pub in_small: f64,
    #[serde(default)]
    pub out_all: f64,
    #[serde(default)]
    pub out_big: f64,
    #[serde(default)]
    pub out_mid: f64,
    #[serde(default)]
    pub out_small: f64,
}

/// 扫描结果字段
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScannerDataRow {
    #[serde(default)]
    pub index: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub data: f64,
}

/// 扫描结果单行
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScannerResultItem {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub base_data_list: Vec<ScannerDataRow>,
    #[serde(default)]
    pub accumulate_data_list: Vec<ScannerDataRow>,
    #[serde(default)]
    pub financial_data_list: Vec<ScannerDataRow>,
    #[serde(default)]
    pub multi_tag_data_list: Vec<ScannerDataRow>,
}

/// 扫描结果
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScannerResult {
    #[serde(default)]
    pub page: i32,
    #[serde(default)]
    pub total_page: i32,
    #[serde(default)]
    pub total_count: i32,
    #[serde(default)]
    pub page_size: i32,
    #[serde(default)]
    pub cursor_id: String,
    #[serde(default)]
    pub items: Vec<ScannerResultItem>,
}

/// 行情权限
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QuotePermission {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub expire_at: i64,
}

// ========== 请求模型 ==========

/// 日级财务数据请求
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct FinancialDailyRequest {
    pub symbols: Vec<String>,
    pub market: String,
    pub fields: Vec<String>,
    pub begin_date: String,
    pub end_date: String,
}

/// 财报数据请求
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct FinancialReportRequest {
    pub symbols: Vec<String>,
    pub market: String,
    pub fields: Vec<String>,
    pub period_type: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub begin_date: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub end_date: String,
}

/// 公司行动请求
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct CorporateActionRequest {
    pub symbols: Vec<String>,
    pub market: String,
    pub action_type: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub begin_date: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub end_date: String,
}

/// 选股扫描请求
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct MarketScannerRequest {
    pub market: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_filter_list: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accumulate_filter_list: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub financial_filter_list: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_tags_filter_list: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_field_data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_tags_fields: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_state_deserialize() {
        let json = r#"{"market":"US","marketStatus":"Trading","status":"TRADING","openTime":"09:30"}"#;
        let m: MarketState = serde_json::from_str(json).unwrap();
        assert_eq!(m.market, "US");
        assert_eq!(m.market_status, "Trading");
        assert_eq!(m.open_time, "09:30");
    }

    #[test]
    fn test_brief_deserialize() {
        let json = r#"{"symbol":"AAPL","latestPrice":150.5,"latestTime":1700000000,"preClose":149.0,"askPrice":150.6,"askSize":100,"volume":1000000,"changeRate":0.01}"#;
        let b: Brief = serde_json::from_str(json).unwrap();
        assert_eq!(b.symbol, "AAPL");
        assert_eq!(b.latest_price, 150.5);
        assert_eq!(b.pre_close, 149.0);
        assert_eq!(b.ask_price, 150.6);
        assert_eq!(b.ask_size, 100);
        assert_eq!(b.change_rate, 0.01);
    }

    #[test]
    fn test_kline_deserialize() {
        let json = r#"{"symbol":"AAPL","period":"day","items":[{"time":1700000000,"open":150.0,"close":151.0,"high":152.0,"low":149.0,"volume":1000}]}"#;
        let k: Kline = serde_json::from_str(json).unwrap();
        assert_eq!(k.symbol, "AAPL");
        assert_eq!(k.period, "day");
        assert_eq!(k.items.len(), 1);
        assert_eq!(k.items[0].open, 150.0);
    }

    #[test]
    fn test_financial_daily_request_serializes_to_snake_case() {
        let req = FinancialDailyRequest {
            symbols: vec!["AAPL".into()],
            market: "US".into(),
            fields: vec!["shares_outstanding".into()],
            begin_date: "2025-01-01".into(),
            end_date: "2025-01-31".into(),
        };
        let v: serde_json::Value = serde_json::to_value(&req).unwrap();
        let obj = v.as_object().unwrap();
        assert!(obj.contains_key("begin_date"));
        assert!(obj.contains_key("end_date"));
        assert!(!obj.contains_key("beginDate"));
        assert!(!obj.contains_key("endDate"));
    }

    #[test]
    fn test_future_kline_request_serializes_to_snake_case() {
        use crate::model::quote_requests::FutureKlineRequest;
        let req = FutureKlineRequest {
            contract_codes: Some(vec!["CL2609".into()]),
            period: Some("day".into()),
            begin_time: Some(-1),
            end_time: Some(-1),
            ..Default::default()
        };
        let v: serde_json::Value = serde_json::to_value(&req).unwrap();
        let obj = v.as_object().unwrap();
        assert!(obj.contains_key("contract_codes"));
        assert!(obj.contains_key("begin_time"));
        assert!(obj.contains_key("end_time"));
        assert!(!obj.contains_key("contractCodes"));
    }

    #[test]
    fn test_market_scanner_request_snake_case() {
        let req = MarketScannerRequest {
            market: "US".into(),
            page: Some(0),
            page_size: Some(10),
            ..Default::default()
        };
        let v = serde_json::to_value(&req).unwrap();
        let obj = v.as_object().unwrap();
        assert!(obj.contains_key("page_size"));
        assert!(!obj.contains_key("pageSize"));
    }

    #[test]
    fn test_depth_deserialize() {
        let json = r#"{"symbol":"AAPL","asks":[{"price":150.0,"count":1,"volume":100}],"bids":[{"price":149.5,"count":1,"volume":200}]}"#;
        let d: Depth = serde_json::from_str(json).unwrap();
        assert_eq!(d.symbol, "AAPL");
        assert_eq!(d.asks.len(), 1);
        assert_eq!(d.asks[0].price, 150.0);
        assert_eq!(d.bids[0].volume, 200);
    }

    #[test]
    fn test_quote_permission_deserialize() {
        let json = r#"[{"name":"usStockQuote","expireAt":1700000000}]"#;
        let ps: Vec<QuotePermission> = serde_json::from_str(json).unwrap();
        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0].name, "usStockQuote");
        assert_eq!(ps[0].expire_at, 1700000000);
    }
}

// ============================================================================
// Batch 3-5：扩展响应模型
// ============================================================================

/// 合约代码 + 名称（all_symbol_names 返回）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SymbolName {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub market: String,
}

/// 交易元数据（quote_stock_trade）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TradeMeta {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub lot_size: i32,
    #[serde(default)]
    pub min_tick: f64,
    #[serde(default)]
    pub spread_scale: f64,
    #[serde(default)]
    pub shortable_flag: String,
    #[serde(default)]
    pub marginable_flag: String,
}

/// 股票详情（stock_detail）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StockDetail {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub name_cn: String,
    #[serde(default)]
    pub name_en: String,
    #[serde(default)]
    pub exchange: String,
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub sec_type: String,
    #[serde(default)]
    pub sector: String,
    #[serde(default)]
    pub industry: String,
    #[serde(default)]
    pub listing_date: i64,
    #[serde(default)]
    pub market_cap: f64,
    #[serde(default)]
    pub circulation_cap: f64,
    #[serde(default)]
    pub total_shares: f64,
    #[serde(default)]
    pub eps_ttm: f64,
    #[serde(default)]
    pub pe_ratio_ttm: f64,
}

/// 做空数据（quote_shortable_stocks）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ShortInterest {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub settlement_date: String,
    #[serde(default)]
    pub short_interest: f64,
    #[serde(default)]
    pub avg_daily_volume: f64,
    #[serde(default)]
    pub days_to_cover: f64,
    #[serde(default)]
    pub percent_of_float: f64,
    #[serde(default)]
    pub short_interest_previous: f64,
    #[serde(default)]
    pub percent_change: f64,
}

/// 经纪商明细。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BrokerDetail {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
}

/// 经纪商档位条目。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StockBrokerItem {
    #[serde(default)]
    pub level: i32,
    #[serde(default)]
    pub price: f64,
    #[serde(default)]
    pub brokers: Vec<BrokerDetail>,
}

/// 经纪商持仓（stock_broker）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StockBroker {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub level_ask_list: Vec<StockBrokerItem>,
    #[serde(default)]
    pub level_bid_list: Vec<StockBrokerItem>,
}

/// 股票基本面（stock_fundamental）。raw map，由调用方进一步解析。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StockFundamental {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub items: Vec<serde_json::Value>,
}

/// 股票行业归属（stock_industry）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StockIndustry {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub gsector: String,
    #[serde(default)]
    pub ggroup: String,
    #[serde(default)]
    pub gind: String,
    #[serde(default)]
    pub gsubind: String,
    #[serde(default)]
    pub level: String,
}

/// 成交榜单条目（trade_rank）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TradeRankItem {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub latest_price: f64,
    #[serde(default)]
    pub change: f64,
    #[serde(default)]
    pub change_rate: f64,
    #[serde(default)]
    pub volume: i64,
    #[serde(default)]
    pub amount: f64,
}

/// K 线配额明细。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KlineQuotaDetail {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub used_bars: i32,
    #[serde(default)]
    pub quota_bars: i32,
    #[serde(default)]
    pub last_access: i64,
}

/// K 线配额（kline_quota）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KlineQuota {
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub used: i32,
    #[serde(default)]
    pub quota: i32,
    #[serde(default)]
    pub detail: Vec<KlineQuotaDetail>,
}

/// 隐含波动率指标（impliedVolMetric）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ImpliedVolMetric {
    #[serde(default)]
    pub period: String,
    #[serde(default)]
    pub percentile: f64,
    #[serde(default)]
    pub rank: f64,
}

/// 期权历史波动率时序点（volatilityList item）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptionVolatilityPoint {
    #[serde(default)]
    pub implied_vol: f64,
    #[serde(default)]
    pub percentile: f64,
    #[serde(default)]
    pub rank: f64,
    #[serde(default)]
    pub his_volatility: f64,
    /// Unix ms timestamp
    #[serde(default)]
    pub timestamp: i64,
}

/// 期权分析（option_analysis）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptionAnalysis {
    #[serde(default)]
    pub symbol: String,
    /// 30 日隐含波动率
    #[serde(default)]
    pub implied_vol30_days: f64,
    /// 历史波动率
    #[serde(default)]
    pub his_volatility: f64,
    /// IV/HV 比率
    #[serde(default)]
    pub iv_his_v_ratio: f64,
    /// 看涨/看跌比率
    #[serde(default)]
    pub call_put_ratio: f64,
    /// 隐含波动率指标（含 period、percentile、rank）
    #[serde(default)]
    pub implied_vol_metric: Option<ImpliedVolMetric>,
    #[serde(default)]
    pub volatility_list: Vec<OptionVolatilityPoint>,
}

/// 期权代码（all_hk_option_symbols 返回）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptionSymbol {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_cn: String,
    #[serde(default)]
    pub name_en: String,
    #[serde(default)]
    pub underlying_symbol: String,
}

/// 期货主力合约历史（future_main_contract）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureMainContractHistory {
    #[serde(default)]
    pub contract_code: String,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub begin_date: String,
    #[serde(default)]
    pub end_date: String,
}

/// 单个期货交易时段。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureTradingSegment {
    #[serde(default)]
    pub start: i64,
    #[serde(default)]
    pub end: i64,
    #[serde(default)]
    pub r#type: String,
}

/// 期货交易时段（future_trading_date）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureTradingTime {
    #[serde(default)]
    pub contract_code: String,
    #[serde(default)]
    pub biz_date: String,
    #[serde(default)]
    pub zone: String,
    #[serde(default)]
    pub trading_times: Vec<FutureTradingSegment>,
}

/// 期货逐笔（future_tick v3.0）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureTradeTickItem {
    #[serde(default)]
    pub contract_code: String,
    #[serde(default)]
    pub index: i64,
    #[serde(default)]
    pub time: i64,
    #[serde(default)]
    pub price: f64,
    #[serde(default)]
    pub volume: i64,
    #[serde(default)]
    pub direction: String,
}

/// 期货盘口（future_depth）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureDepth {
    #[serde(default)]
    pub contract_code: String,
    #[serde(default)]
    pub timestamp: i64,
    #[serde(default)]
    pub asks: Vec<DepthLevel>,
    #[serde(default)]
    pub bids: Vec<DepthLevel>,
}

/// 基金合约信息（fund_contracts）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FundContractInfo {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub fund_type: String,
    #[serde(default)]
    pub inception: String,
    #[serde(default)]
    pub net_asset_value: f64,
    #[serde(default)]
    pub expense_ratio: f64,
}

/// 基金净值报价（fund_quote）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FundQuote {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub latest_nav: f64,
    #[serde(default)]
    pub change: f64,
    #[serde(default)]
    pub change_rate: f64,
    #[serde(default)]
    pub date: String,
}

/// 基金历史净值（fund_history_quote）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FundHistoryQuote {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub nav: f64,
}

/// 窝轮简要信息（warrant_briefs）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WarrantBrief {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub latest_price: f64,
    #[serde(default)]
    pub change: f64,
    #[serde(default)]
    pub change_rate: f64,
    #[serde(default)]
    pub volume: i64,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub underlying: String,
    #[serde(default)]
    pub issuer: String,
    #[serde(default)]
    pub expiry_date: String,
    #[serde(default)]
    pub strike_price: f64,
    #[serde(default)]
    pub warrant_type: String,
}

/// 窝轮筛选结果（warrant_filter）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WarrantFilterResult {
    #[serde(default)]
    pub total: i32,
    #[serde(default)]
    pub items: Vec<WarrantBrief>,
    #[serde(default)]
    pub page_size: i32,
    #[serde(default)]
    pub page: i32,
}

/// 行业列表条目（industry_list）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IndustryItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub level: String,
}

/// 行业归属股票条目（industry_stock_list）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IndustryStock {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub industry_id: String,
    #[serde(default)]
    pub change: f64,
    #[serde(default)]
    pub change_rate: f64,
}

/// 汇率数据（financial_exchange_rate）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRate {
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub rate: f64,
    #[serde(default)]
    pub base_currency: String,
}

/// 财报货币（financial_currency）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FinancialCurrency {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub currency: String,
}

/// 交易日历（trading_calendar）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TradingCalendarItem {
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub is_trading: bool,
    #[serde(default)]
    pub session_type: String,
}

/// 扫描器标签条目（market_scanner_tags `tagList` 数组元素）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MarketScannerTag {
    #[serde(default)]
    pub field: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub values: Vec<String>,
}

/// 扫描器标签分组（market_scanner_tags 顶层数组元素）。
/// 服务端 wire: `[{market, multiTagField, tagList:[...]}]`
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MarketScannerTagGroup {
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub multi_tag_field: String,
    #[serde(default)]
    pub tag_list: Vec<MarketScannerTag>,
}

/// 兼容别名，保留旧名称以防外部代码直接引用。
#[deprecated(since = "0.5.4", note = "use MarketScannerTagGroup instead")]
pub type MarketScannerTags = MarketScannerTagGroup;

/// 隔夜行情（quote_overnight）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QuoteOvernight {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub pre_close: f64,
    #[serde(default)]
    pub open: f64,
    #[serde(default)]
    pub close: f64,
    #[serde(default)]
    pub high: f64,
    #[serde(default)]
    pub low: f64,
    #[serde(default)]
    pub volume: i64,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub change: f64,
    #[serde(default)]
    pub change_rate: f64,
    #[serde(default)]
    pub begin_time: i64,
    #[serde(default)]
    pub end_time: i64,
}

