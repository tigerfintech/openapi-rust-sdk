//! Quote client module, wrapping all quote-related APIs.

use serde_json::Value;
use crate::client::api_request::ApiRequest;
use crate::client::http_client::HttpClient;
use crate::error::TigerError;

/// API version constants matching Python SDK
const VERSION_V1: &str = "1.0";
const VERSION_V3: &str = "3.0";

/// Quote client wrapping all quote-related APIs.
/// Holds an HttpClient reference, sends requests via execute_request, returns the data field.
pub struct QuoteClient<'a> {
    http_client: &'a HttpClient,
}

impl<'a> QuoteClient<'a> {
    /// Create a new quote client
    pub fn new(http_client: &'a HttpClient) -> Self {
        Self { http_client }
    }

    /// Internal helper: build request, send, return data field (default version)
    async fn execute(&self, method: &str, params: Value) -> Result<Option<Value>, TigerError> {
        let biz_content = serde_json::to_string(&params)
            .map_err(|e| TigerError::Config(format!("failed to serialize biz params: {}", e)))?;
        let req = ApiRequest::new(method, biz_content);
        let resp = self.http_client.execute_request(&req).await?;
        Ok(resp.data)
    }

    /// Internal helper: build request with specific API version
    async fn execute_with_version(&self, method: &str, params: Value, version: &str) -> Result<Option<Value>, TigerError> {
        let biz_content = serde_json::to_string(&params)
            .map_err(|e| TigerError::Config(format!("failed to serialize biz params: {}", e)))?;
        let req = ApiRequest::with_version(method, biz_content, version);
        let resp = self.http_client.execute_request(&req).await?;
        Ok(resp.data)
    }

    // === Basic quote methods ===

    /// Get market state
    pub async fn market_state(&self, market: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"market": market});
        self.execute("market_state", params).await
    }

    /// Get market state (compatibility alias)
    pub async fn get_market_state(&self, market: &str) -> Result<Option<Value>, TigerError> {
        self.market_state(market).await
    }

    /// Get real-time quotes
    pub async fn quote_real_time(&self, symbols: &[&str]) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbols": symbols});
        self.execute("quote_real_time", params).await
    }

    /// Get stock briefs (compatibility alias)
    pub async fn get_brief(&self, symbols: &[&str]) -> Result<Option<Value>, TigerError> {
        self.quote_real_time(symbols).await
    }

    /// Get K-line data
    pub async fn kline(&self, symbol: &str, period: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbols": [symbol], "period": period});
        self.execute("kline", params).await
    }

    /// Get K-line data (compatibility alias)
    pub async fn get_kline(&self, symbol: &str, period: &str) -> Result<Option<Value>, TigerError> {
        self.kline(symbol, period).await
    }

    /// Get intraday timeline data
    pub async fn timeline(&self, symbols: &[&str]) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbols": symbols});
        self.execute_with_version("timeline", params, VERSION_V3).await
    }

    /// Get timeline (compatibility alias)
    pub async fn get_timeline(&self, symbols: &[&str]) -> Result<Option<Value>, TigerError> {
        self.timeline(symbols).await
    }

    /// Get trade tick data
    pub async fn trade_tick(&self, symbols: &[&str]) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbols": symbols});
        self.execute("trade_tick", params).await
    }

    /// Get trade tick (compatibility alias)
    pub async fn get_trade_tick(&self, symbols: &[&str]) -> Result<Option<Value>, TigerError> {
        self.trade_tick(symbols).await
    }

    /// Get depth quote
    pub async fn quote_depth(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol});
        self.execute("quote_depth", params).await
    }

    /// Get depth quote (compatibility alias)
    pub async fn get_quote_depth(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        self.quote_depth(symbol).await
    }

    // === Option quote methods ===

    /// Get option expiration dates.
    /// Uses `symbols` array format matching Python SDK.
    pub async fn option_expiration(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbols": [symbol]});
        self.execute("option_expiration", params).await
    }

    /// Get option expiration (compatibility alias)
    pub async fn get_option_expiration(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        self.option_expiration(symbol).await
    }

    /// Get option chain with version 3.0 (matching Python SDK).
    /// Uses `contracts` array format with symbol/expiry objects.
    pub async fn option_chain(&self, symbol: &str, expiry: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({
            "contracts": [{"symbol": symbol, "expiry": expiry}]
        });
        self.execute_with_version("option_chain", params, VERSION_V3).await
    }

    /// Get option chain (compatibility alias)
    pub async fn get_option_chain(&self, symbol: &str, expiry: &str) -> Result<Option<Value>, TigerError> {
        self.option_chain(symbol, expiry).await
    }

    /// Get option brief quotes.
    /// Uses `option_basics` array format with parsed identifier fields.
    pub async fn option_brief(&self, identifiers: &[&str]) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"identifiers": identifiers});
        self.execute("option_brief", params).await
    }

    /// Get option brief (compatibility alias)
    pub async fn get_option_brief(&self, identifiers: &[&str]) -> Result<Option<Value>, TigerError> {
        self.option_brief(identifiers).await
    }

    /// Get option K-line data
    pub async fn option_kline(&self, identifier: &str, period: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"identifier": identifier, "period": period});
        self.execute("option_kline", params).await
    }

    /// Get option K-line (compatibility alias)
    pub async fn get_option_kline(&self, identifier: &str, period: &str) -> Result<Option<Value>, TigerError> {
        self.option_kline(identifier, period).await
    }

    // === Futures quote methods ===

    /// Get futures exchange list
    pub async fn future_exchange(&self) -> Result<Option<Value>, TigerError> {
        self.execute("future_exchange", serde_json::json!({"sec_type": "FUT"})).await
    }

    /// Get futures exchange (compatibility alias)
    pub async fn get_future_exchange(&self) -> Result<Option<Value>, TigerError> {
        self.future_exchange().await
    }

    /// Get futures contract list
    pub async fn future_contracts(&self, exchange: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"exchange": exchange});
        self.execute("future_contracts", params).await
    }

    /// Get futures contracts (compatibility alias)
    pub async fn get_future_contracts(&self, exchange: &str) -> Result<Option<Value>, TigerError> {
        self.future_contracts(exchange).await
    }

    /// Get futures real-time quotes
    pub async fn future_real_time_quote(&self, symbols: &[&str]) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbols": symbols});
        self.execute("future_real_time_quote", params).await
    }

    /// Get futures real-time quote (compatibility alias)
    pub async fn get_future_real_time_quote(&self, symbols: &[&str]) -> Result<Option<Value>, TigerError> {
        self.future_real_time_quote(symbols).await
    }

    /// Get futures K-line data
    pub async fn future_kline(&self, symbol: &str, period: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol, "period": period});
        self.execute("future_kline", params).await
    }

    /// Get futures K-line (compatibility alias)
    pub async fn get_future_kline(&self, symbol: &str, period: &str) -> Result<Option<Value>, TigerError> {
        self.future_kline(symbol, period).await
    }

    // === Fundamental and capital flow methods ===

    /// Get financial daily data
    pub async fn financial_daily(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol});
        self.execute("financial_daily", params).await
    }

    /// Get financial daily (compatibility alias)
    pub async fn get_financial_daily(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        self.financial_daily(symbol).await
    }

    /// Get financial report
    pub async fn financial_report(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol});
        self.execute("financial_report", params).await
    }

    /// Get financial report (compatibility alias)
    pub async fn get_financial_report(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        self.financial_report(symbol).await
    }

    /// Get corporate action
    pub async fn corporate_action(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol});
        self.execute("corporate_action", params).await
    }

    /// Get corporate action (compatibility alias)
    pub async fn get_corporate_action(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        self.corporate_action(symbol).await
    }

    /// Get capital flow
    pub async fn capital_flow(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol});
        self.execute("capital_flow", params).await
    }

    /// Get capital flow (compatibility alias)
    pub async fn get_capital_flow(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        self.capital_flow(symbol).await
    }

    /// Get capital distribution
    pub async fn capital_distribution(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"symbol": symbol});
        self.execute("capital_distribution", params).await
    }

    /// Get capital distribution (compatibility alias)
    pub async fn get_capital_distribution(&self, symbol: &str) -> Result<Option<Value>, TigerError> {
        self.capital_distribution(symbol).await
    }

    // === Scanner and quote permission methods ===

    /// Market scanner (uses version 1.0 matching Python SDK)
    pub async fn market_scanner(&self, params: Value) -> Result<Option<Value>, TigerError> {
        self.execute_with_version("market_scanner", params, VERSION_V1).await
    }

    /// Get quote permission
    pub async fn grab_quote_permission(&self) -> Result<Option<Value>, TigerError> {
        self.execute("grab_quote_permission", serde_json::json!({})).await
    }
}

#[cfg(test)]
mod tests;
