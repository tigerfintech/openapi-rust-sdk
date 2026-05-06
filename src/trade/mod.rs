//! 交易客户端，封装所有交易相关 API。
//!
//! 所有方法返回强类型响应（[`Order`]、[`Position`]、[`Asset`]、[`PreviewResult`] 等）。

use serde::Serialize;
use serde_json::Value;

use crate::client::api_request::ApiRequest;
use crate::client::http_client::HttpClient;
use crate::error::TigerError;
use crate::model::contract::Contract;
use crate::model::order::{Order, OrderRequest};
use crate::model::position::Position;
use crate::model::trade::{
    Asset, OrderIdResult, PlaceOrderResult, PreviewResult, PrimeAsset, Transaction,
};

/// 交易客户端
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

    /// call_into 的 Option 版本，没数据时返回 None 而不是 T::default()。
    async fn call_optional<T, P>(&self, method: &str, params: P) -> Result<Option<T>, TigerError>
    where
        T: serde::de::DeserializeOwned,
        P: Serialize,
    {
        let biz = serde_json::to_string(&params)
            .map_err(|e| TigerError::Config(format!("serialize biz params failed: {}", e)))?;
        let req = ApiRequest::new(method, biz);
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

    /// 剥掉服务端 `{"items":[...]}` 外包装
    async fn call_into_items<T, P>(&self, method: &str, params: P) -> Result<Vec<T>, TigerError>
    where
        T: serde::de::DeserializeOwned,
        P: Serialize,
    {
        let biz = serde_json::to_string(&params)
            .map_err(|e| TigerError::Config(format!("serialize biz params failed: {}", e)))?;
        let req = ApiRequest::new(method, biz);
        let resp = self.http_client.execute_request(&req).await?;
        let Some(v) = resp.data else { return Ok(Vec::new()); };
        if v.is_null() {
            return Ok(Vec::new());
        }
        // 尝试从对象中取 items
        let raw = match &v {
            Value::String(s) => serde_json::from_str::<Value>(s).unwrap_or(v.clone()),
            _ => v.clone(),
        };
        if let Value::Object(map) = &raw {
            if let Some(items) = map.get("items") {
                if items.is_null() {
                    return Ok(Vec::new());
                }
                return serde_json::from_value::<Vec<T>>(items.clone())
                    .map_err(|e| TigerError::Config(format!("decode items failed: {}", e)));
            }
        }
        // 回退：data 本身就是数组
        if raw.is_array() {
            return serde_json::from_value::<Vec<T>>(raw)
                .map_err(|e| TigerError::Config(format!("decode array data failed: {}", e)));
        }
        Ok(Vec::new())
    }

    // ========== 合约查询 ==========

    /// 查询单个合约
    pub async fn get_contract(&self, symbol: &str, sec_type: &str) -> Result<Vec<Contract>, TigerError> {
        self.call_into_items(
            "contract",
            serde_json::json!({
                "account": self.account,
                "symbol": symbol,
                "sec_type": sec_type,
            }),
        )
        .await
    }

    /// 批量查询合约
    pub async fn get_contracts(
        &self,
        symbols: &[&str],
        sec_type: &str,
    ) -> Result<Vec<Contract>, TigerError> {
        self.call_into_items(
            "contracts",
            serde_json::json!({
                "account": self.account,
                "symbols": symbols,
                "sec_type": sec_type,
            }),
        )
        .await
    }

    /// 查询衍生品合约（OPT/WAR/IOPT）。`expiry` 必填（如 "20260619"）。
    pub async fn get_quote_contract(
        &self,
        symbol: &str,
        sec_type: &str,
        expiry: &str,
    ) -> Result<Vec<Contract>, TigerError> {
        self.call_into_items(
            "quote_contract",
            serde_json::json!({
                "account": self.account,
                "symbols": [symbol],
                "sec_type": sec_type,
                "expiry": expiry,
            }),
        )
        .await
    }

    // ========== 订单操作 ==========

    /// 预览订单
    pub async fn preview_order(
        &self,
        order: OrderRequest,
    ) -> Result<Option<PreviewResult>, TigerError> {
        let mut order = order;
        order.account = Some(self.account.clone());
        self.call_optional("preview_order", order).await
    }

    /// 下单
    pub async fn place_order(
        &self,
        order: OrderRequest,
    ) -> Result<Option<PlaceOrderResult>, TigerError> {
        let mut order = order;
        order.account = Some(self.account.clone());
        self.call_optional("place_order", order).await
    }

    /// 修改订单
    pub async fn modify_order(
        &self,
        id: i64,
        order: OrderRequest,
    ) -> Result<Option<OrderIdResult>, TigerError> {
        let mut order = order;
        order.account = Some(self.account.clone());
        order.id = Some(id);
        self.call_optional("modify_order", order).await
    }

    /// 取消订单
    pub async fn cancel_order(&self, id: i64) -> Result<Option<OrderIdResult>, TigerError> {
        self.call_optional(
            "cancel_order",
            serde_json::json!({ "account": self.account, "id": id }),
        )
        .await
    }

    // ========== 订单查询 ==========

    /// 查询全部订单
    pub async fn get_orders(&self) -> Result<Vec<Order>, TigerError> {
        self.call_into_items("orders", serde_json::json!({ "account": self.account })).await
    }

    /// 查询待成交订单
    pub async fn get_active_orders(&self) -> Result<Vec<Order>, TigerError> {
        self.call_into_items("active_orders", serde_json::json!({ "account": self.account })).await
    }

    /// 查询已撤销订单
    pub async fn get_inactive_orders(&self) -> Result<Vec<Order>, TigerError> {
        self.call_into_items("inactive_orders", serde_json::json!({ "account": self.account })).await
    }

    /// 查询已成交订单。`start_ms` / `end_ms` 为 13 位毫秒时间戳，必填。
    pub async fn get_filled_orders(
        &self,
        start_ms: i64,
        end_ms: i64,
    ) -> Result<Vec<Order>, TigerError> {
        self.call_into_items(
            "filled_orders",
            serde_json::json!({
                "account": self.account,
                "start_date": start_ms,
                "end_date": end_ms,
            }),
        )
        .await
    }

    /// 查询订单成交明细。`symbol` 和 `sec_type` 必填；服务端 key 为 `order_id`。
    pub async fn get_order_transactions(
        &self,
        order_id: i64,
        symbol: &str,
        sec_type: &str,
    ) -> Result<Vec<Transaction>, TigerError> {
        self.call_into_items(
            "order_transactions",
            serde_json::json!({
                "account": self.account,
                "order_id": order_id,
                "symbol": symbol,
                "sec_type": sec_type,
            }),
        )
        .await
    }

    // ========== 持仓和资产 ==========

    /// 查询持仓
    pub async fn get_positions(&self) -> Result<Vec<Position>, TigerError> {
        self.call_into_items("positions", serde_json::json!({ "account": self.account })).await
    }

    /// 查询资产
    pub async fn get_assets(&self) -> Result<Vec<Asset>, TigerError> {
        self.call_into_items("assets", serde_json::json!({ "account": self.account })).await
    }

    /// 查询综合账户资产（不裹 items）
    pub async fn get_prime_assets(&self) -> Result<Option<PrimeAsset>, TigerError> {
        self.call_optional("prime_assets", serde_json::json!({ "account": self.account })).await
    }
}

fn decode_value<T>(v: Value) -> Result<T, TigerError>
where
    T: serde::de::DeserializeOwned,
{
    match serde_json::from_value::<T>(v.clone()) {
        Ok(out) => Ok(out),
        Err(_) => {
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

#[cfg(test)]
mod tests;
