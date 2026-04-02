//! 行情查询客户端模块，封装所有行情相关 API。

use serde_json::Value;
use crate::client::api_request::ApiRequest;
use crate::client::http_client::HttpClient;
use crate::error::TigerError;

/// 行情查询客户端，封装所有行情相关 API。
/// 持有 HttpClient 引用，通过 execute_request 发送请求，返回 data 字段。
pub struct QuoteClient<'a> {
    http_client: &'a HttpClient,
}

impl<'a> QuoteClient<'a> {
    /// 创建行情查询客户端
    pub fn new(http_client: &'a HttpClient) -> Self {
        Self { http_client }
    }

    /// 内部通用方法：构造请求、发送、返回 data 字段
    async fn execute(&self, method: &str, params: Value) -> Result<Option<Value>, TigerError> {
        let biz_content = serde_json::to_string(&params)
            .map_err(|e| TigerError::Config(format!("序列化业务参数失败: {}", e)))?;
        let req = ApiRequest::new(method, biz_content);
        let resp = self.http_client.execute_request(&req).await?;
        Ok(resp.data)
    }

    // === 基础行情方法 ===

    /// 获取市场状态
    pub async fn market_state(&self, market: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"market": market});
        self.execute("market_state", params).await
    }

    /// 获取实时报价
    pub async fn quote_real_time(&self, symbols: &[&str]) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbols": symbols});
        self.execute("quote_real_time", params).await
    }

    /// 获取 K 线数据
    pub async fn kline(&self, symbol: &str, period: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbols": [symbol], "period": period});
        self.execute("kline", params).await
    }

    /// 获取分时数据
    pub async fn timeline(&self, symbols: &[&str]) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbols": symbols});
        self.execute("timeline", params).await
    }

    /// 获取逐笔成交数据
    pub async fn trade_tick(&self, symbols: &[&str]) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbols": symbols});
        self.execute("trade_tick", params).await
    }

    /// 获取深度行情
    pub async fn quote_depth(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol});
        self.execute("quote_depth", params).await
    }

    // === 期权行情方法 ===

    /// 获取期权到期日
    pub async fn option_expiration(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbols": [symbol]});
        self.execute("option_expiration", params).await
    }

    /// 获取期权链
    pub async fn option_chain(&self, symbol: &str, expiry: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol, "expiry": expiry});
        self.execute("option_chain", params).await
    }

    /// 获取期权报价
    pub async fn option_brief(&self, identifiers: &[&str]) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"identifiers": identifiers});
        self.execute("option_brief", params).await
    }

    /// 获取期权 K 线
    pub async fn option_kline(&self, identifier: &str, period: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"identifier": identifier, "period": period});
        self.execute("option_kline", params).await
    }

    // === 期货行情方法 ===

    /// 获取期货交易所列表
    pub async fn future_exchange(&self) -> Result<Option<Value>, TigerError> {
        self.execute("future_exchange", serde_json::json!({"sec_type": "FUT"})).await
    }

    /// 获取期货合约列表
    pub async fn future_contracts(&self, exchange: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"exchange": exchange});
        self.execute("future_contracts", params).await
    }

    /// 获取期货实时报价
    pub async fn future_real_time_quote(&self, symbols: &[&str]) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbols": symbols});
        self.execute("future_real_time_quote", params).await
    }

    /// 获取期货 K 线
    pub async fn future_kline(&self, symbol: &str, period: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol, "period": period});
        self.execute("future_kline", params).await
    }

    // === 基本面和资金流向方法 ===

    /// 获取财务日报
    pub async fn financial_daily(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol});
        self.execute("financial_daily", params).await
    }

    /// 获取财务报告
    pub async fn financial_report(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol});
        self.execute("financial_report", params).await
    }

    /// 获取公司行动
    pub async fn corporate_action(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol});
        self.execute("corporate_action", params).await
    }

    /// 获取资金流向
    pub async fn capital_flow(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol});
        self.execute("capital_flow", params).await
    }

    /// 获取资金分布
    pub async fn capital_distribution(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol});
        self.execute("capital_distribution", params).await
    }

    // === 选股器和行情权限方法 ===

    /// 选股器
    pub async fn market_scanner(&self, params: Value) -> Result<Option<Value>, TigerError> {
        self.execute("market_scanner", params).await
    }

    /// 获取行情权限
    pub async fn grab_quote_permission(&self) -> Result<Option<Value>, TigerError> {
        self.execute("grab_quote_permission", serde_json::json!({})).await
    }
}

#[cfg(test)]
mod tests;
