//! 行情客户端，封装所有行情相关 API。
//!
//! 所有方法返回强类型响应（[`MarketState`]、[`Brief`]、[`Kline`] 等），
//! 请求参数使用 snake_case 序列化（匹配服务端约定）。

use serde::Serialize;
use serde_json::Value;

use crate::client::api_request::ApiRequest;
use crate::client::http_client::HttpClient;
use crate::error::TigerError;
use crate::model::quote::{
    Brief, CapitalDistribution, CapitalFlow, CorporateAction, CorporateActionRequest, Depth,
    FinancialDailyItem, FinancialDailyRequest, FinancialReportItem, FinancialReportRequest,
    FutureContractInfo, FutureExchange, FutureKline, FutureKlineRequest, FutureQuote, Kline,
    MarketScannerRequest, MarketState, OptionBrief, OptionChain, OptionExpiration, OptionKline,
    QuotePermission, ScannerResult, Timeline, TradeTick,
};

/// API 版本常量
const VERSION_V1: &str = "1.0";
const VERSION_V2: &str = "2.0";
const VERSION_V3: &str = "3.0";

/// 行情客户端，封装所有行情 API。
pub struct QuoteClient<'a> {
    http_client: &'a HttpClient,
}

impl<'a> QuoteClient<'a> {
    /// 创建行情客户端。调用方通常使用 [`HttpClient::with_quote_server`] 以便
    /// 请求发往 `config.quote_server_url`。
    pub fn new(http_client: &'a HttpClient) -> Self {
        Self { http_client }
    }

    /// 内部通用：构造请求、发送、把 data 解析为 T。
    async fn call_into<T, P>(&self, method: &str, params: P) -> Result<T, TigerError>
    where
        T: serde::de::DeserializeOwned + Default,
        P: Serialize,
    {
        self.call_into_versioned(method, params, None).await
    }

    async fn call_into_versioned<T, P>(
        &self,
        method: &str,
        params: P,
        version: Option<&str>,
    ) -> Result<T, TigerError>
    where
        T: serde::de::DeserializeOwned + Default,
        P: Serialize,
    {
        let biz = serde_json::to_string(&params)
            .map_err(|e| TigerError::Config(format!("serialize biz params failed: {}", e)))?;
        let req = match version {
            Some(v) => ApiRequest::with_version(method, biz, v),
            None => ApiRequest::new(method, biz),
        };
        let resp = self.http_client.execute_request(&req).await?;
        unmarshal_data(resp.data)
    }

    // ========== 基础行情 ==========

    /// 获取市场状态
    pub async fn get_market_state(&self, market: &str) -> Result<Vec<MarketState>, TigerError> {
        self.call_into("market_state", serde_json::json!({ "market": market })).await
    }

    /// 获取实时快照
    pub async fn get_brief(&self, symbols: &[&str]) -> Result<Vec<Brief>, TigerError> {
        self.call_into("brief", serde_json::json!({ "symbols": symbols })).await
    }

    /// 获取 K 线（一个标的一组）
    pub async fn get_kline(&self, symbol: &str, period: &str) -> Result<Vec<Kline>, TigerError> {
        self.call_into("kline", serde_json::json!({ "symbols": [symbol], "period": period })).await
    }

    /// 获取分时数据（v3.0）
    pub async fn get_timeline(&self, symbols: &[&str]) -> Result<Vec<Timeline>, TigerError> {
        self.call_into_versioned("timeline", serde_json::json!({ "symbols": symbols }), Some(VERSION_V3)).await
    }

    /// 获取逐笔成交
    pub async fn get_trade_tick(&self, symbols: &[&str]) -> Result<Vec<TradeTick>, TigerError> {
        self.call_into("trade_tick", serde_json::json!({ "symbols": symbols })).await
    }

    /// 获取深度报价。`market` 必填（例如 "US" / "HK"）。
    pub async fn get_quote_depth(&self, symbol: &str, market: &str) -> Result<Vec<Depth>, TigerError> {
        self.call_into(
            "quote_depth",
            serde_json::json!({ "symbols": [symbol], "market": market }),
        )
        .await
    }

    // ========== 期权行情 ==========

    /// 获取期权到期日
    pub async fn get_option_expiration(&self, symbol: &str) -> Result<Vec<OptionExpiration>, TigerError> {
        self.call_into(
            "option_expiration",
            serde_json::json!({ "symbols": [symbol] }),
        )
        .await
    }

    /// 获取期权链（v3.0）。`expiry` 格式为 "YYYY-MM-DD"，内部转为毫秒时间戳。
    pub async fn get_option_chain(
        &self,
        symbol: &str,
        expiry: &str,
    ) -> Result<Vec<OptionChain>, TigerError> {
        let expiry_ts = parse_expiry_to_ms(expiry)?;
        let params = serde_json::json!({
            "option_basic": [{ "symbol": symbol, "expiry": expiry_ts }]
        });
        self.call_into_versioned("option_chain", params, Some(VERSION_V3)).await
    }

    /// 获取期权快照（v2.0）。`identifiers` 为 OCC 格式（如 "AAPL 240119C00150000"）。
    pub async fn get_option_brief(
        &self,
        identifiers: &[&str],
    ) -> Result<Vec<OptionBrief>, TigerError> {
        let mut option_basics: Vec<serde_json::Value> = Vec::with_capacity(identifiers.len());
        for id in identifiers {
            let contract = parse_option_identifier(id)?;
            option_basics.push(serde_json::json!({
                "symbol": contract.symbol,
                "expiry": contract.expiry,
                "right": contract.right,
                "strike": contract.strike,
            }));
        }
        let params = serde_json::json!({ "option_basic": option_basics });
        self.call_into_versioned("option_brief", params, Some(VERSION_V2)).await
    }

    /// 获取期权 K 线（v2.0）。
    pub async fn get_option_kline(
        &self,
        identifier: &str,
        period: &str,
    ) -> Result<Vec<OptionKline>, TigerError> {
        let contract = parse_option_identifier(identifier)?;
        let params = serde_json::json!({
            "option_query": [{
                "symbol": contract.symbol,
                "expiry": contract.expiry,
                "right": contract.right,
                "strike": contract.strike,
                "period": period,
            }]
        });
        self.call_into_versioned("option_kline", params, Some(VERSION_V2)).await
    }

    // ========== 期货行情 ==========

    /// 获取期货交易所列表
    pub async fn get_future_exchange(&self) -> Result<Vec<FutureExchange>, TigerError> {
        self.call_into("future_exchange", serde_json::json!({ "sec_type": "FUT" })).await
    }

    /// 获取期货合约列表
    pub async fn get_future_contracts(
        &self,
        exchange_code: &str,
    ) -> Result<Vec<FutureContractInfo>, TigerError> {
        self.call_into(
            "future_contract_by_exchange_code",
            serde_json::json!({ "exchange_code": exchange_code }),
        )
        .await
    }

    /// 获取期货实时报价
    pub async fn get_future_real_time_quote(
        &self,
        contract_codes: &[&str],
    ) -> Result<Vec<FutureQuote>, TigerError> {
        self.call_into(
            "future_real_time_quote",
            serde_json::json!({ "contract_codes": contract_codes }),
        )
        .await
    }

    /// 获取期货 K 线
    pub async fn get_future_kline(
        &self,
        mut req: FutureKlineRequest,
    ) -> Result<Vec<FutureKline>, TigerError> {
        if req.begin_time == 0 {
            req.begin_time = -1;
        }
        if req.end_time == 0 {
            req.end_time = -1;
        }
        self.call_into("future_kline", req).await
    }

    // ========== 基本面和资金流向 ==========

    /// 获取日级财务数据
    pub async fn get_financial_daily(
        &self,
        req: FinancialDailyRequest,
    ) -> Result<Vec<FinancialDailyItem>, TigerError> {
        self.call_into("financial_daily", req).await
    }

    /// 获取财报数据
    pub async fn get_financial_report(
        &self,
        req: FinancialReportRequest,
    ) -> Result<Vec<FinancialReportItem>, TigerError> {
        self.call_into("financial_report", req).await
    }

    /// 获取公司行动。服务端返回 `{symbol: [...]}` 结构，内部扁平化为单个数组。
    pub async fn get_corporate_action(
        &self,
        req: CorporateActionRequest,
    ) -> Result<Vec<CorporateAction>, TigerError> {
        let grouped: std::collections::BTreeMap<String, Vec<CorporateAction>> =
            self.call_into("corporate_action", req).await?;
        let mut out = Vec::new();
        for (_, mut list) in grouped {
            out.append(&mut list);
        }
        Ok(out)
    }

    /// 获取资金流向
    pub async fn get_capital_flow(
        &self,
        symbol: &str,
        market: &str,
        period: &str,
    ) -> Result<Option<CapitalFlow>, TigerError> {
        self.call_optional(
            "capital_flow",
            serde_json::json!({ "symbol": symbol, "market": market, "period": period }),
        )
        .await
    }

    /// 获取资金分布
    pub async fn get_capital_distribution(
        &self,
        symbol: &str,
        market: &str,
    ) -> Result<Option<CapitalDistribution>, TigerError> {
        self.call_optional(
            "capital_distribution",
            serde_json::json!({ "symbol": symbol, "market": market }),
        )
        .await
    }

    // ========== 选股扫描和行情权限 ==========

    /// 选股扫描（v1.0）
    pub async fn market_scanner(
        &self,
        req: MarketScannerRequest,
    ) -> Result<Option<ScannerResult>, TigerError> {
        self.call_optional_versioned("market_scanner", req, Some(VERSION_V1)).await
    }

    /// 获取行情权限
    pub async fn grab_quote_permission(&self) -> Result<Vec<QuotePermission>, TigerError> {
        self.call_into("grab_quote_permission", serde_json::json!({})).await
    }

    // ========== 内部辅助 ==========

    async fn call_optional<T, P>(&self, method: &str, params: P) -> Result<Option<T>, TigerError>
    where
        T: serde::de::DeserializeOwned,
        P: Serialize,
    {
        self.call_optional_versioned(method, params, None).await
    }

    async fn call_optional_versioned<T, P>(
        &self,
        method: &str,
        params: P,
        version: Option<&str>,
    ) -> Result<Option<T>, TigerError>
    where
        T: serde::de::DeserializeOwned,
        P: Serialize,
    {
        let biz = serde_json::to_string(&params)
            .map_err(|e| TigerError::Config(format!("serialize biz params failed: {}", e)))?;
        let req = match version {
            Some(v) => ApiRequest::with_version(method, biz, v),
            None => ApiRequest::new(method, biz),
        };
        let resp = self.http_client.execute_request(&req).await?;
        match resp.data {
            None => Ok(None),
            Some(v) if v.is_null() => Ok(None),
            Some(v) => {
                let parsed: T = decode_value(v)?;
                Ok(Some(parsed))
            }
        }
    }
}

/// 把 `data` 字段解码为目标类型 T，兼容服务端偶发双重编码（字符串包 JSON）。
/// 当 data 为 None / Null 时返回 T::default()。
fn unmarshal_data<T>(data: Option<Value>) -> Result<T, TigerError>
where
    T: serde::de::DeserializeOwned + Default,
{
    match data {
        None => Ok(T::default()),
        Some(v) if v.is_null() => Ok(T::default()),
        Some(v) => decode_value(v),
    }
}

fn decode_value<T>(v: Value) -> Result<T, TigerError>
where
    T: serde::de::DeserializeOwned,
{
    // 先按原值解码
    match serde_json::from_value::<T>(v.clone()) {
        Ok(out) => Ok(out),
        Err(_) => {
            // 如果 data 是 JSON 字符串（双重编码），先解一层
            if let Value::String(s) = &v {
                return serde_json::from_str::<T>(s).map_err(|e| {
                    TigerError::Config(format!("decode data (double-encoded) failed: {}", e))
                });
            }
            serde_json::from_value::<T>(v)
                .map_err(|e| TigerError::Config(format!("decode data failed: {}", e)))
        }
    }
}

// ========== 期权 identifier 辅助 ==========

struct OptionContract {
    symbol: String,
    /// 毫秒时间戳
    expiry: i64,
    /// "CALL" 或 "PUT"
    right: String,
    strike: f64,
}

/// 解析 "YYYY-MM-DD" 为 UTC 毫秒时间戳
fn parse_expiry_to_ms(expiry: &str) -> Result<i64, TigerError> {
    use chrono::NaiveDate;
    let d = NaiveDate::parse_from_str(expiry, "%Y-%m-%d")
        .map_err(|e| TigerError::Config(format!("invalid expiry {:?}: expected YYYY-MM-DD: {}", expiry, e)))?;
    let dt = d
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| TigerError::Config(format!("invalid expiry date: {:?}", expiry)))?;
    let utc = dt.and_utc();
    Ok(utc.timestamp_millis())
}

/// 解析 OCC 期权 identifier（"AAPL 240119C00150000"）
fn parse_option_identifier(identifier: &str) -> Result<OptionContract, TigerError> {
    let trimmed = identifier.trim();
    let mut it = trimmed.splitn(2, ' ');
    let symbol = it
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| TigerError::Config(format!("invalid option identifier: {:?}", identifier)))?
        .to_string();
    let rest = it
        .next()
        .map(str::trim)
        .ok_or_else(|| TigerError::Config(format!("invalid option identifier: {:?}", identifier)))?;

    if rest.len() < 15 {
        return Err(TigerError::Config(format!("option code too short: {:?}", rest)));
    }

    let date_str = &rest[..6];
    let date = chrono::NaiveDate::parse_from_str(date_str, "%y%m%d")
        .map_err(|_| TigerError::Config(format!("invalid date in identifier: {:?}", date_str)))?;
    let expiry = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| TigerError::Config("invalid date".into()))?
        .and_utc()
        .timestamp_millis();

    let right_char = rest.as_bytes()[6];
    let right = match right_char {
        b'C' => "CALL",
        b'P' => "PUT",
        other => {
            return Err(TigerError::Config(format!(
                "invalid right character: {:?}",
                other as char
            )));
        }
    }
    .to_string();

    let strike_str = &rest[7..];
    let mut strike_int: i64 = 0;
    for c in strike_str.chars() {
        if !c.is_ascii_digit() {
            return Err(TigerError::Config(format!("invalid strike digits: {:?}", strike_str)));
        }
        strike_int = strike_int * 10 + (c as i64 - '0' as i64);
    }
    let strike = strike_int as f64 / 1000.0;

    Ok(OptionContract {
        symbol,
        expiry,
        right,
        strike,
    })
}

#[cfg(test)]
mod tests;
