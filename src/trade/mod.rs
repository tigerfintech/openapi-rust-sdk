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
    AggregateAssets, AnalyticsAsset, Asset, EstimateTradableQuantity, ForexOrderResult,
    FundDetails, FundingHistoryItem, ManagedAccount, OrderIdResult, PlaceOrderResult,
    PositionTransferDetail, PositionTransferExternalRecord, PositionTransferRecord, PreviewResult,
    PrimeAsset, SegmentFund, SegmentFundHistoryItem, Transaction,
};
use crate::model::trade_requests::{
    AggregateAssetsRequest, AnalyticsAssetRequest, AssetsRequest, DerivativeContractsRequest,
    EstimateTradableQuantityRequest, ForexOrderRequest, FundDetailsRequest, FundingHistoryRequest,
    GetOrderRequest, ManagedAccountsRequest, OrderTransactionsRequest, OrdersRequest,
    PositionTransferDetailRequest, PositionTransferExternalRecordsRequest,
    PositionTransferRecordsRequest, PositionTransferRequest, PositionsRequest, SegmentFundRequest,
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
    pub async fn get_orders(&self, req: OrdersRequest) -> Result<Vec<Order>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_into_items("orders", req).await
    }

    /// 查询待成交订单
    pub async fn get_active_orders(&self, req: OrdersRequest) -> Result<Vec<Order>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_into_items("active_orders", req).await
    }

    /// 查询已撤销订单
    pub async fn get_inactive_orders(&self, req: OrdersRequest) -> Result<Vec<Order>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_into_items("inactive_orders", req).await
    }

    /// 查询已成交订单。`req.start_date` / `req.end_date` 为 13 位毫秒时间戳。
    pub async fn get_filled_orders(&self, req: OrdersRequest) -> Result<Vec<Order>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_into_items("filled_orders", req).await
    }

    /// 按订单 ID 查询单个订单。id 和 order_id 都为 0 时返回 None。
    pub async fn get_order(&self, req: GetOrderRequest) -> Result<Option<Order>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        // id 和 order_id 都未设置时无意义
        if req.id.unwrap_or(0) == 0 && req.order_id.unwrap_or(0) == 0 {
            return Ok(None);
        }
        self.call_optional("order_no", req).await
    }

    /// 查询订单成交明细。
    pub async fn get_order_transactions(
        &self,
        req: OrderTransactionsRequest,
    ) -> Result<Vec<Transaction>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_into_items("order_transactions", req).await
    }

    // ========== 持仓和资产 ==========

    /// 查询持仓
    pub async fn get_positions(&self, req: PositionsRequest) -> Result<Vec<Position>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_into_items("positions", req).await
    }

    /// 查询资产
    pub async fn get_assets(&self, req: AssetsRequest) -> Result<Vec<Asset>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_into_items("assets", req).await
    }

    /// 查询综合账户资产（不裹 items）
    pub async fn get_prime_assets(&self, req: AssetsRequest) -> Result<Option<PrimeAsset>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_optional("prime_assets", req).await
    }

    // ========== 账户管理 ==========

    /// 查询机构子账户列表（wire: accounts）
    pub async fn get_managed_accounts(
        &self,
        req: ManagedAccountsRequest,
    ) -> Result<Vec<ManagedAccount>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_into_items("accounts", req).await
    }

    /// 查询衍生品合约列表（wire: quote_contract）
    pub async fn get_derivative_contracts(
        &self,
        req: DerivativeContractsRequest,
    ) -> Result<Vec<Contract>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_into_items("quote_contract", req).await
    }

    /// 查询资产分析（按日，wire: analytics_asset）
    pub async fn get_analytics_asset(
        &self,
        req: AnalyticsAssetRequest,
    ) -> Result<Vec<AnalyticsAsset>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_into_items("analytics_asset", req).await
    }

    /// 查询综合资产汇总（wire: aggregate_assets）
    pub async fn get_aggregate_assets(
        &self,
        req: AggregateAssetsRequest,
    ) -> Result<Option<AggregateAssets>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_optional("aggregate_assets", req).await
    }

    /// 估算可交易数量（wire: estimate_tradable_quantity）
    pub async fn get_estimate_tradable_quantity(
        &self,
        req: EstimateTradableQuantityRequest,
    ) -> Result<Option<EstimateTradableQuantity>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_optional("estimate_tradable_quantity", req).await
    }

    // ========== 外汇 ==========

    /// 外汇下单（wire: place_forex_order）
    pub async fn place_forex_order(
        &self,
        req: ForexOrderRequest,
    ) -> Result<Option<ForexOrderResult>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_optional("place_forex_order", req).await
    }

    // ========== 资金调拨 ==========

    /// 查询可调拨金额（wire: segment_fund_available，返回数组）
    pub async fn get_segment_fund_available(
        &self,
        req: SegmentFundRequest,
    ) -> Result<Vec<SegmentFund>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_into_items("segment_fund_available", req).await
    }

    /// 查询调拨历史（wire: segment_fund_history，裸数组，无 items 包装）
    pub async fn get_segment_fund_history(
        &self,
        req: SegmentFundRequest,
    ) -> Result<Vec<SegmentFundHistoryItem>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_into_items("segment_fund_history", req).await
    }

    /// 执行资金调拨（wire: transfer_segment_fund）
    pub async fn transfer_segment_fund(
        &self,
        req: SegmentFundRequest,
    ) -> Result<Option<SegmentFund>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_optional("transfer_segment_fund", req).await
    }

    /// 撤销资金调拨（wire: cancel_segment_fund）
    pub async fn cancel_segment_fund(
        &self,
        req: SegmentFundRequest,
    ) -> Result<Option<SegmentFund>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_optional("cancel_segment_fund", req).await
    }

    // ========== 资金明细 ==========

    /// 查询资金明细（wire: fund_details，`{items:[]}` 外壳）
    pub async fn get_fund_details(
        &self,
        req: FundDetailsRequest,
    ) -> Result<Vec<FundDetails>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_into_items("fund_details", req).await
    }

    /// 查询调拨记录（wire: transfer_fund，`{items:[]}` 外壳）
    pub async fn get_funding_history(
        &self,
        req: FundingHistoryRequest,
    ) -> Result<Vec<FundingHistoryItem>, TigerError> {
        let mut req = req;
        if req.account.is_none() {
            req.account = Some(self.account.clone());
        }
        self.call_into_items("transfer_fund", req).await
    }

    // ========== 持仓转移 ==========

    /// 内部转股（wire: position_transfer）
    pub async fn transfer_position(
        &self,
        req: PositionTransferRequest,
    ) -> Result<Option<PositionTransferRecord>, TigerError> {
        self.call_optional("position_transfer", req).await
    }

    /// 查询内部转股记录（wire: position_transfer_records，`{items:[]}` 外壳）
    pub async fn get_position_transfer_records(
        &self,
        req: PositionTransferRecordsRequest,
    ) -> Result<Vec<PositionTransferRecord>, TigerError> {
        let mut req = req;
        if req.account_id.is_none() {
            req.account_id = Some(self.account.clone());
        }
        self.call_into_items("position_transfer_records", req).await
    }

    /// 查询内部转股详情（wire: position_transfer_detail）
    pub async fn get_position_transfer_detail(
        &self,
        req: PositionTransferDetailRequest,
    ) -> Result<Option<PositionTransferDetail>, TigerError> {
        let mut req = req;
        if req.account_id.is_none() {
            req.account_id = Some(self.account.clone());
        }
        self.call_optional("position_transfer_detail", req).await
    }

    /// 查询外部转股记录（wire: position_transfer_external_records，`{items:[]}` 外壳）
    pub async fn get_position_transfer_external_records(
        &self,
        req: PositionTransferExternalRecordsRequest,
    ) -> Result<Vec<PositionTransferExternalRecord>, TigerError> {
        let mut req = req;
        if req.account_id.is_none() {
            req.account_id = Some(self.account.clone());
        }
        self.call_into_items("position_transfer_external_records", req).await
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
