//! 交易客户端模块，封装所有交易相关 API。

use serde_json::Value;
use crate::client::api_request::ApiRequest;
use crate::client::http_client::HttpClient;
use crate::error::TigerError;

/// 交易客户端，封装所有交易相关 API。
/// 持有 HttpClient 引用和交易账户，通过 execute_request 发送请求，返回 data 字段。
pub struct TradeClient<'a> {
    http_client: &'a HttpClient,
    account: String,
}

impl<'a> TradeClient<'a> {
    /// 创建交易客户端
    pub fn new(http_client: &'a HttpClient, account: impl Into<String>) -> Self {
        Self {
            http_client,
            account: account.into(),
        }
    }

    /// 内部通用方法：构造请求、发送、返回 data 字段
    async fn execute(&self, method: &str, params: Value) -> Result<Option<Value>, TigerError> {
        let biz_content = serde_json::to_string(&params)
            .map_err(|e| TigerError::Config(format!("序列化业务参数失败: {}", e)))?;
        let req = ApiRequest::new(method, biz_content);
        let resp = self.http_client.execute_request(&req).await?;
        Ok(resp.data)
    }

    // === 合约查询方法 ===

    /// 查询单个合约
    pub async fn contract(&self, symbol: &str, sec_type: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({
            "account": self.account,
            "symbol": symbol,
            "secType": sec_type,
        });
        self.execute("contract", params).await
    }

    /// 批量查询合约
    pub async fn contracts(&self, symbols: &[&str], sec_type: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({
            "account": self.account,
            "symbols": symbols,
            "secType": sec_type,
        });
        self.execute("contracts", params).await
    }

    /// 查询衍生品合约
    pub async fn quote_contract(&self, symbol: &str, sec_type: &str) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({
            "account": self.account,
            "symbol": symbol,
            "secType": sec_type,
        });
        self.execute("quote_contract", params).await
    }

    // === 订单操作方法 ===

    /// 下单
    pub async fn place_order(&self, order: Value) -> Result<Option<Value>, TigerError> {
        let mut params = order;
        params["account"] = serde_json::json!(self.account);
        self.execute("place_order", params).await
    }

    /// 预览订单
    pub async fn preview_order(&self, order: Value) -> Result<Option<Value>, TigerError> {
        let mut params = order;
        params["account"] = serde_json::json!(self.account);
        self.execute("preview_order", params).await
    }

    /// 修改订单
    pub async fn modify_order(&self, id: i64, order: Value) -> Result<Option<Value>, TigerError> {
        let mut params = order;
        params["account"] = serde_json::json!(self.account);
        params["id"] = serde_json::json!(id);
        self.execute("modify_order", params).await
    }

    /// 取消订单
    pub async fn cancel_order(&self, id: i64) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({
            "account": self.account,
            "id": id,
        });
        self.execute("cancel_order", params).await
    }

    // === 订单查询方法 ===

    /// 查询全部订单
    pub async fn orders(&self) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"account": self.account});
        self.execute("orders", params).await
    }

    /// 查询待成交订单
    pub async fn active_orders(&self) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"account": self.account});
        self.execute("active_orders", params).await
    }

    /// 查询已撤销订单
    pub async fn inactive_orders(&self) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"account": self.account});
        self.execute("inactive_orders", params).await
    }

    /// 查询已成交订单
    pub async fn filled_orders(&self) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"account": self.account});
        self.execute("filled_orders", params).await
    }

    // === 持仓和资产查询方法 ===

    /// 查询持仓
    pub async fn positions(&self) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"account": self.account});
        self.execute("positions", params).await
    }

    /// 查询资产
    pub async fn assets(&self) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"account": self.account});
        self.execute("assets", params).await
    }

    /// 查询综合账户资产
    pub async fn prime_assets(&self) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({"account": self.account});
        self.execute("prime_assets", params).await
    }

    /// 查询订单成交明细
    pub async fn order_transactions(&self, id: i64) -> Result<Option<Value>, TigerError> {
        let params = serde_json::json!({
            "account": self.account,
            "id": id,
        });
        self.execute("order_transactions", params).await
    }
}

#[cfg(test)]
mod tests;
