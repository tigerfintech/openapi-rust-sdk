//! 交易请求参数结构体。
//!
//! 字段命名规则：
//! - snake_case（与 wire 真名对齐）
//! - 全部可选，加 `#[serde(skip_serializing_if = "Option::is_none")]`
//! - struct 级别不加 rename_all（请求字段本身就是 snake_case）

use serde::{Deserialize, Serialize};

/// TransferItem — 内部转股单项（用于 PositionTransferRequest.transfers）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransferItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
}

/// OrdersRequest — 查询订单列表。
/// wire methods: orders / active_orders / inactive_orders / filled_orders
#[derive(Debug, Clone, Serialize, Default)]
pub struct OrdersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// ms 时间戳
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<i64>,
    /// ms 时间戳
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_brief: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub states: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seg_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
    /// 仅 active_orders 使用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i64>,
}

/// GetOrderRequest — 按订单 ID 查询单个订单。
/// wire method: order_no
#[derive(Debug, Clone, Serialize, Default)]
pub struct GetOrderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// 全局订单 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// 账户维度订单 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_brief: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_charges: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// OrderTransactionsRequest — 查询订单成交明细。
/// wire method: order_transactions
#[derive(Debug, Clone, Serialize, Default)]
pub struct OrderTransactionsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    /// ms 时间戳
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<i64>,
    /// ms 时间戳
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put_call: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

/// PositionsRequest — 查询持仓。
/// wire method: positions
#[derive(Debug, Clone, Serialize, Default)]
pub struct PositionsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_accounts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_quote_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// AssetsRequest — 查询资产。
/// wire methods: assets / prime_assets
#[derive(Debug, Clone, Serialize, Default)]
pub struct AssetsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_accounts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_value: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// ManagedAccountsRequest — 查询机构子账户列表。
/// wire method: accounts
#[derive(Debug, Clone, Serialize, Default)]
pub struct ManagedAccountsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// DerivativeContractsRequest — 查询衍生品合约列表。
/// wire method: derivative_contracts
#[derive(Debug, Clone, Serialize, Default)]
pub struct DerivativeContractsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// AnalyticsAssetRequest — 查询资产分析（按日）。
/// wire method: analytics_asset
/// 注意：start_date/end_date 是字符串格式 "YYYY-MM-DD"，与 OrdersRequest 不同。
#[derive(Debug, Clone, Serialize, Default)]
pub struct AnalyticsAssetRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seg_type: Option<String>,
    /// 格式 "YYYY-MM-DD"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    /// 格式 "YYYY-MM-DD"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// AggregateAssetsRequest — 查询综合资产（base_currency 视角汇总）。
/// wire method: aggregate_assets
#[derive(Debug, Clone, Serialize, Default)]
pub struct AggregateAssetsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seg_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// EstimateTradableQuantityRequest — 估算可交易数量。
/// wire method: estimate_tradable_quantity
#[derive(Debug, Clone, Serialize, Default)]
pub struct EstimateTradableQuantityRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// ForexOrderRequest — 外汇下单。
/// wire method: place_forex_order
#[derive(Debug, Clone, Serialize, Default)]
pub struct ForexOrderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seg_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// SegmentFundRequest — 子账户资金调拨（available/history/transfer/cancel 共用）。
/// wire methods: segment_fund_available / segment_fund_history / transfer_segment_fund / cancel_segment_fund
#[derive(Debug, Clone, Serialize, Default)]
pub struct SegmentFundRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_segment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_segment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// FundDetailsRequest — 资金明细。
/// wire method: fund_details
///
/// **Wire types (0.6 breaking):** `start_date` / `end_date` are yyyy-MM-dd
/// strings, not epoch-ms integers. Java's FundDetailsModel exposes them
/// as `String startDate` / `String endDate`. Previously typed as
/// `Option<i64>` here, which caused the gateway to reject with
/// "parse 'start_date' error, supported format: 'yyyy-MM-dd'".
#[derive(Debug, Clone, Serialize, Default)]
pub struct FundDetailsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seg_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fund_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// 起始日期(yyyy-MM-dd)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    /// 结束日期(yyyy-MM-dd)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// FundingHistoryRequest — 资金调拨历史。
/// wire method: transfer_fund
#[derive(Debug, Clone, Serialize, Default)]
pub struct FundingHistoryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seg_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// ms 时间戳
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<i64>,
    /// ms 时间戳
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// PositionTransferRequest — 内部转股（跨账户换仓）。
/// wire method: position_transfer
#[derive(Debug, Clone, Serialize, Default)]
pub struct PositionTransferRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfers: Option<Vec<TransferItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// PositionTransferRecordsRequest — 内部转股记录查询。
/// wire method: position_transfer_records
/// 注意：账户字段 wire 名为 account_id（不是 account）；TradeClient 会自动注入。
#[derive(Debug, Clone, Serialize, Default)]
pub struct PositionTransferRecordsRequest {
    /// 账户 ID（wire 字段名 "account_id"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// 格式 "YYYY-MM-DD"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_date: Option<String>,
    /// 格式 "YYYY-MM-DD"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// PositionTransferDetailRequest — 内部转股详情（按 ID）。
/// wire method: position_transfer_detail
#[derive(Debug, Clone, Serialize, Default)]
pub struct PositionTransferDetailRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// PositionTransferExternalRecordsRequest — 外部转股记录查询。
/// wire method: position_transfer_external_records
/// 参数字段与 PositionTransferRecordsRequest 相同。
#[derive(Debug, Clone, Serialize, Default)]
pub struct PositionTransferExternalRecordsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}
/// OptionExerciseCheckRequest — 行权检验请求。
/// wire method: option_exercise_check
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionExerciseCheckRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    /// 期权合约 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<i64>,
    /// Exercise | Expire
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub exercise_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    /// yyyy-MM-dd，Exercise 类型建议填
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executing_date: Option<String>,
    /// Exercise 类型建议填
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_force: Option<bool>,
    /// 0–10，Expire 类型专用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itm_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// OptionExercisePositionRequest — 查询可行权持仓请求。
/// wire method: option_exercise_position
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionExercisePositionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    /// Exercise | Expire
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub exercise_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// OptionExerciseSubmitRequest — 提交行权申请请求。
/// wire method: option_exercise_submit
/// Exercise 类型：executing_date 和 is_force 为必填。
/// Expire 类型：itm_rate 可选（0–10）。
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionExerciseSubmitRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<i64>,
    /// Exercise | Expire
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub exercise_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    /// Exercise 必填，yyyy-MM-dd
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executing_date: Option<String>,
    /// Exercise 必填
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_force: Option<bool>,
    /// 0–10，Expire 专用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itm_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// OptionExerciseRecordsRequest — 分页查询行权记录请求。
/// wire method: option_exercise_record
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionExerciseRecordsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    /// 从 1 开始，默认 1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
    /// 1–100，默认 20
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i32>,
    /// New | Cancel | Success | Fail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Exercise | Expire
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub exercise_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// symbol | expire_date | strike | is_call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// OptionExerciseCancelRequest — 撤销行权申请请求。
/// wire method: option_exercise_cancel
#[derive(Debug, Clone, Serialize, Default)]
pub struct OptionExerciseCancelRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_json<T: Serialize>(v: &T) -> serde_json::Value {
        serde_json::to_value(v).unwrap()
    }

    // ── Default serialization: all-None → "{}" ──

    #[test]
    fn test_default_requests_serialize_to_empty_object() {
        assert_eq!(to_json(&OrdersRequest::default()), serde_json::json!({}));
        assert_eq!(to_json(&GetOrderRequest::default()), serde_json::json!({}));
        assert_eq!(
            to_json(&OrderTransactionsRequest::default()),
            serde_json::json!({})
        );
        assert_eq!(to_json(&PositionsRequest::default()), serde_json::json!({}));
        assert_eq!(to_json(&AssetsRequest::default()), serde_json::json!({}));
        assert_eq!(
            to_json(&ManagedAccountsRequest::default()),
            serde_json::json!({})
        );
        assert_eq!(
            to_json(&DerivativeContractsRequest::default()),
            serde_json::json!({})
        );
        assert_eq!(
            to_json(&AnalyticsAssetRequest::default()),
            serde_json::json!({})
        );
        assert_eq!(
            to_json(&AggregateAssetsRequest::default()),
            serde_json::json!({})
        );
        assert_eq!(
            to_json(&EstimateTradableQuantityRequest::default()),
            serde_json::json!({})
        );
        assert_eq!(
            to_json(&ForexOrderRequest::default()),
            serde_json::json!({})
        );
        assert_eq!(
            to_json(&SegmentFundRequest::default()),
            serde_json::json!({})
        );
        assert_eq!(
            to_json(&FundDetailsRequest::default()),
            serde_json::json!({})
        );
        assert_eq!(
            to_json(&FundingHistoryRequest::default()),
            serde_json::json!({})
        );
        assert_eq!(
            to_json(&PositionTransferRequest::default()),
            serde_json::json!({})
        );
        assert_eq!(
            to_json(&OptionExerciseCheckRequest::default()),
            serde_json::json!({})
        );
        assert_eq!(
            to_json(&OptionExerciseSubmitRequest::default()),
            serde_json::json!({})
        );
        assert_eq!(
            to_json(&OptionExerciseRecordsRequest::default()),
            serde_json::json!({})
        );
        assert_eq!(
            to_json(&OptionExerciseCancelRequest::default()),
            serde_json::json!({})
        );
    }

    // ── Field name assertions ──

    #[test]
    fn test_orders_request_field_names() {
        let req = OrdersRequest {
            account: Some("DU123".into()),
            sec_type: Some("STK".into()),
            market: Some("US".into()),
            symbol: Some("AAPL".into()),
            start_date: Some(1000),
            end_date: Some(2000),
            limit: Some(50),
            is_brief: Some(true),
            states: Some(vec!["Filled".into()]),
            sort_by: Some("LATEST_CREATED".into()),
            seg_type: Some("SEC".into()),
            lang: Some("en_US".into()),
            page_token: Some("tok".into()),
            parent_id: Some(99),
        };
        let j = to_json(&req);
        assert_eq!(j["account"], "DU123");
        assert_eq!(j["sec_type"], "STK");
        assert_eq!(j["market"], "US");
        assert_eq!(j["symbol"], "AAPL");
        assert_eq!(j["start_date"], 1000);
        assert_eq!(j["end_date"], 2000);
        assert_eq!(j["limit"], 50);
        assert_eq!(j["is_brief"], true);
        assert_eq!(j["states"][0], "Filled");
        assert_eq!(j["sort_by"], "LATEST_CREATED");
        assert_eq!(j["seg_type"], "SEC");
        assert_eq!(j["page_token"], "tok");
        assert_eq!(j["parent_id"], 99);
    }

    #[test]
    fn test_positions_request_field_names() {
        let req = PositionsRequest {
            account: Some("DU123".into()),
            sec_type: Some("STK".into()),
            currency: Some("USD".into()),
            market: Some("US".into()),
            symbol: Some("AAPL".into()),
            sub_accounts: Some(vec!["sub1".into()]),
            expiry: Some("2024-01-19".into()),
            strike: Some("150.0".into()),
            right: Some("CALL".into()),
            asset_quote_type: Some("last".into()),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["currency"], "USD");
        assert_eq!(j["sub_accounts"][0], "sub1");
        assert_eq!(j["asset_quote_type"], "last");
    }

    #[test]
    fn test_assets_request_field_names() {
        let req = AssetsRequest {
            account: Some("DU123".into()),
            sub_accounts: Some(vec!["sub1".into()]),
            segment: Some(true),
            market_value: Some(false),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["segment"], true);
        assert_eq!(j["market_value"], false);
    }

    #[test]
    fn test_forex_order_request_field_names() {
        let req = ForexOrderRequest {
            account: Some("DU123".into()),
            seg_type: Some("SEC".into()),
            source_currency: Some("USD".into()),
            target_currency: Some("HKD".into()),
            source_amount: Some(100.0),
            target_amount: Some(780.0),
            order_type: Some("LMT".into()),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["source_currency"], "USD");
        assert_eq!(j["target_currency"], "HKD");
        assert_eq!(j["source_amount"], 100.0);
        assert_eq!(j["target_amount"], 780.0);
    }

    #[test]
    fn test_segment_fund_request_field_names() {
        let req = SegmentFundRequest {
            account: Some("DU123".into()),
            id: Some("abc".into()),
            from_segment: Some("SEC".into()),
            to_segment: Some("FUT".into()),
            currency: Some("USD".into()),
            amount: Some(500.0),
            limit: Some(10),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["from_segment"], "SEC");
        assert_eq!(j["to_segment"], "FUT");
        assert_eq!(j["amount"], 500.0);
    }

    #[test]
    fn test_position_transfer_request_field_names() {
        let req = PositionTransferRequest {
            from_account: Some("DU1".into()),
            to_account: Some("DU2".into()),
            market: Some("US".into()),
            transfers: Some(vec![TransferItem {
                symbol: Some("AAPL".into()),
                quantity: Some(100),
                sec_type: Some("STK".into()),
                ..Default::default()
            }]),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["from_account"], "DU1");
        assert_eq!(j["to_account"], "DU2");
        assert_eq!(j["transfers"][0]["symbol"], "AAPL");
        assert_eq!(j["transfers"][0]["quantity"], 100);
    }

    #[test]
    fn test_transfer_item_field_names() {
        let item = TransferItem {
            symbol: Some("AAPL".into()),
            quantity: Some(10),
            expiry: Some("2024-01-19".into()),
            strike: Some("150.0".into()),
            right: Some("CALL".into()),
            sec_type: Some("OPT".into()),
        };
        let j = to_json(&item);
        assert_eq!(j["symbol"], "AAPL");
        assert_eq!(j["quantity"], 10);
        assert_eq!(j["expiry"], "2024-01-19");
        assert_eq!(j["strike"], "150.0");
        assert_eq!(j["right"], "CALL");
        assert_eq!(j["sec_type"], "OPT");
    }

    #[test]
    fn test_option_exercise_check_request_type_rename() {
        // exercise_type field serializes as "type" on the wire
        let req = OptionExerciseCheckRequest {
            account: Some("DU1".into()),
            secret_key: Some("secret".into()),
            contract_id: Some(12345),
            exercise_type: Some("Exercise".into()),
            quantity: Some(1.0),
            executing_date: Some("2024-01-19".into()),
            is_force: Some(true),
            itm_rate: Some(5),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        // rename = "type" → wire key is "type", not "exercise_type"
        assert_eq!(j["type"], "Exercise");
        assert_eq!(j["secret_key"], "secret");
        assert_eq!(j["contract_id"], 12345);
        assert_eq!(j["quantity"], 1.0);
        assert_eq!(j["is_force"], true);
        assert_eq!(j["itm_rate"], 5);
    }

    #[test]
    fn test_option_exercise_submit_request_type_rename() {
        let req = OptionExerciseSubmitRequest {
            account: Some("DU1".into()),
            secret_key: Some("secret".into()),
            contract_id: Some(999),
            exercise_type: Some("Expire".into()),
            quantity: Some(2.0),
            itm_rate: Some(3),
            lang: Some("en_US".into()),
            ..Default::default()
        };
        let j = to_json(&req);
        assert_eq!(j["type"], "Expire");
        assert_eq!(j["contract_id"], 999);
        assert_eq!(j["itm_rate"], 3);
        // exercising_date and is_force are None → skipped
        assert!(j.get("executing_date").is_none());
        assert!(j.get("is_force").is_none());
    }

    #[test]
    fn test_option_exercise_records_request_type_rename() {
        let req = OptionExerciseRecordsRequest {
            account: Some("DU1".into()),
            secret_key: Some("secret".into()),
            page: Some(1),
            size: Some(20),
            status: Some("Success".into()),
            exercise_type: Some("Exercise".into()),
            symbol: Some("AAPL".into()),
            order_by: Some("symbol".into()),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["type"], "Exercise");
        assert_eq!(j["page"], 1);
        assert_eq!(j["size"], 20);
        assert_eq!(j["status"], "Success");
        assert_eq!(j["order_by"], "symbol");
    }

    #[test]
    fn test_option_exercise_cancel_request_field_names() {
        let req = OptionExerciseCancelRequest {
            account: Some("DU1".into()),
            secret_key: Some("secret".into()),
            id: Some(42),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["id"], 42);
        assert_eq!(j["secret_key"], "secret");
    }

    #[test]
    fn test_position_transfer_records_request_field_names() {
        let req = PositionTransferRecordsRequest {
            account_id: Some("DU1".into()),
            since_date: Some("2024-01-01".into()),
            to_date: Some("2024-01-31".into()),
            market: Some("US".into()),
            limit: Some(50),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["account_id"], "DU1");
        assert_eq!(j["since_date"], "2024-01-01");
        assert_eq!(j["to_date"], "2024-01-31");
    }

    #[test]
    fn test_estimate_tradable_quantity_request_field_names() {
        let req = EstimateTradableQuantityRequest {
            account: Some("DU1".into()),
            symbol: Some("AAPL".into()),
            sec_type: Some("STK".into()),
            action: Some("BUY".into()),
            order_type: Some("MKT".into()),
            limit_price: Some(150.5),
            market: Some("US".into()),
            currency: Some("USD".into()),
            expiry: Some("2024-01-19".into()),
            strike: Some("150.0".into()),
            right: Some("CALL".into()),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["action"], "BUY");
        assert_eq!(j["order_type"], "MKT");
        assert_eq!(j["limit_price"], 150.5);
    }

    #[test]
    fn test_analytics_asset_request_field_names() {
        let req = AnalyticsAssetRequest {
            account: Some("DU1".into()),
            seg_type: Some("SEC".into()),
            start_date: Some("2024-01-01".into()),
            end_date: Some("2024-01-31".into()),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["seg_type"], "SEC");
        assert_eq!(j["start_date"], "2024-01-01");
        assert_eq!(j["end_date"], "2024-01-31");
    }

    #[test]
    fn test_aggregate_assets_request_field_names() {
        let req = AggregateAssetsRequest {
            account: Some("DU1".into()),
            base_currency: Some("USD".into()),
            seg_type: Some("SEC".into()),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["base_currency"], "USD");
    }

    #[test]
    fn test_derivative_contracts_request_field_names() {
        let req = DerivativeContractsRequest {
            account: Some("DU1".into()),
            symbols: Some(vec!["AAPL".into()]),
            sec_type: Some("OPT".into()),
            expiry: Some("2024-01-19".into()),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["symbols"][0], "AAPL");
        assert_eq!(j["expiry"], "2024-01-19");
    }

    #[test]
    fn test_fund_details_request_field_names() {
        let req = FundDetailsRequest {
            account: Some("DU1".into()),
            seg_types: Some(vec!["SEC".into()]),
            fund_type: Some("deposit".into()),
            currency: Some("USD".into()),
            start_date: Some("2024-01-01".into()),
            end_date: Some("2024-01-31".into()),
            limit: Some(50),
            page_token: Some("tok".into()),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["seg_types"][0], "SEC");
        assert_eq!(j["fund_type"], "deposit");
        assert_eq!(j["page_token"], "tok");
        // 0.6 breaking-change fix: start_date/end_date must serialize as
        // yyyy-MM-dd strings, not epoch-ms integers.
        assert_eq!(
            j["start_date"], "2024-01-01",
            "start_date must be a date string (0.6 wire fix)"
        );
        assert_eq!(
            j["end_date"], "2024-01-31",
            "end_date must be a date string (0.6 wire fix)"
        );
    }

    #[test]
    fn test_get_order_request_field_names() {
        let req = GetOrderRequest {
            account: Some("DU1".into()),
            id: Some(12345),
            order_id: Some(67890),
            is_brief: Some(true),
            show_charges: Some(true),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["id"], 12345);
        assert_eq!(j["order_id"], 67890);
        assert_eq!(j["show_charges"], true);
    }

    #[test]
    fn test_order_transactions_request_field_names() {
        let req = OrderTransactionsRequest {
            account: Some("DU1".into()),
            order_id: Some(123),
            symbol: Some("AAPL".into()),
            sec_type: Some("STK".into()),
            start_date: Some(1000),
            end_date: Some(2000),
            limit: Some(10),
            expiry: Some("2024-01-19".into()),
            strike: Some(150.0),
            put_call: Some("CALL".into()),
            lang: Some("en_US".into()),
            page_token: Some("tok".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["order_id"], 123);
        assert_eq!(j["put_call"], "CALL");
        assert_eq!(j["strike"], 150.0);
    }

    #[test]
    fn test_funding_history_request_field_names() {
        let req = FundingHistoryRequest {
            account: Some("DU1".into()),
            seg_type: Some("SEC".into()),
            currency: Some("USD".into()),
            start_date: Some(1000),
            end_date: Some(2000),
            limit: Some(50),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["seg_type"], "SEC");
    }

    #[test]
    fn test_managed_accounts_request_field_names() {
        let req = ManagedAccountsRequest {
            account: Some("DU1".into()),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["account"], "DU1");
    }

    #[test]
    fn test_option_exercise_position_request_type_rename() {
        let req = OptionExercisePositionRequest {
            account: Some("DU1".into()),
            secret_key: Some("secret".into()),
            exercise_type: Some("Exercise".into()),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["type"], "Exercise");
    }

    #[test]
    fn test_position_transfer_detail_request_field_names() {
        let req = PositionTransferDetailRequest {
            account_id: Some("DU1".into()),
            id: Some("abc".into()),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["account_id"], "DU1");
        assert_eq!(j["id"], "abc");
    }

    #[test]
    fn test_position_transfer_external_records_request_field_names() {
        let req = PositionTransferExternalRecordsRequest {
            account_id: Some("DU1".into()),
            since_date: Some("2024-01-01".into()),
            to_date: Some("2024-01-31".into()),
            market: Some("US".into()),
            limit: Some(50),
            lang: Some("en_US".into()),
        };
        let j = to_json(&req);
        assert_eq!(j["account_id"], "DU1");
        assert_eq!(j["since_date"], "2024-01-01");
    }
}
