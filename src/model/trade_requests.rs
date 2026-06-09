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
    /// ms 时间戳
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<i64>,
    /// ms 时间戳
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<i64>,
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
