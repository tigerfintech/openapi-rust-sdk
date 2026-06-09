//! 交易响应模型：Asset、PrimeAsset、PreviewResult、PlaceOrderResult、OrderIdResult、Transaction。

use serde::Deserialize;
use serde_json::Value;

/// 资产分段（securities / commodities 等）
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AssetSegment {
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub net_liquidation: f64,
    #[serde(default)]
    pub cash_value: f64,
    #[serde(default)]
    pub available_funds: f64,
    #[serde(default)]
    pub equity_with_loan: f64,
    #[serde(default)]
    pub excess_liquidity: f64,
    #[serde(default)]
    pub accrued_cash: f64,
    #[serde(default)]
    pub accrued_dividend: f64,
    #[serde(default)]
    pub init_margin_req: f64,
    #[serde(default)]
    pub maint_margin_req: f64,
    #[serde(default)]
    pub gross_position_value: f64,
    #[serde(default)]
    pub leverage: f64,
}

/// 账户资产条目（来自 /assets）
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub capability: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub buying_power: f64,
    #[serde(default)]
    pub cash_value: f64,
    #[serde(default)]
    pub net_liquidation: f64,
    #[serde(default, rename = "realizedPnL")]
    pub realized_pnl: f64,
    #[serde(default, rename = "unrealizedPnL")]
    pub unrealized_pnl: f64,
    #[serde(default)]
    pub segments: Vec<AssetSegment>,
}

/// 分币种资产明细
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyAsset {
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub cash_balance: f64,
    #[serde(default)]
    pub cash_available_for_trade: f64,
    #[serde(default)]
    pub forex_rate: f64,
}

/// 综合账户分段
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PrimeAssetSegment {
    #[serde(default)]
    pub capability: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub cash_balance: f64,
    #[serde(default)]
    pub cash_available_for_trade: f64,
    #[serde(default)]
    pub gross_position_value: f64,
    #[serde(default)]
    pub equity_with_loan: f64,
    #[serde(default)]
    pub net_liquidation: f64,
    #[serde(default)]
    pub init_margin: f64,
    #[serde(default)]
    pub maintain_margin: f64,
    #[serde(default)]
    pub overnight_margin: f64,
    #[serde(default, rename = "unrealizedPL")]
    pub unrealized_pl: f64,
    #[serde(default, rename = "unrealizedPLByCostOfCarry")]
    pub unrealized_pl_by_cost_of_carry: f64,
    #[serde(default, rename = "realizedPL")]
    pub realized_pl: f64,
    #[serde(default, rename = "totalTodayPL")]
    pub total_today_pl: f64,
    #[serde(default)]
    pub excess_liquidation: f64,
    #[serde(default)]
    pub overnight_liquidation: f64,
    #[serde(default)]
    pub buying_power: f64,
    #[serde(default)]
    pub locked_funds: f64,
    #[serde(default)]
    pub leverage: f64,
    #[serde(default)]
    pub uncollected: f64,
    #[serde(default)]
    pub currency_assets: Vec<CurrencyAsset>,
    #[serde(default)]
    pub consolidated_seg_types: Vec<String>,
}

/// 综合账户资产
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PrimeAsset {
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub update_timestamp: i64,
    #[serde(default)]
    pub segments: Vec<PrimeAssetSegment>,
}

/// 订单预览结果
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResult {
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub is_pass: bool,
    #[serde(default)]
    pub commission: f64,
    #[serde(default)]
    pub commission_currency: String,
    #[serde(default)]
    pub margin_currency: String,
    #[serde(default)]
    pub init_margin: f64,
    #[serde(default)]
    pub init_margin_before: f64,
    #[serde(default)]
    pub maint_margin: f64,
    #[serde(default)]
    pub maint_margin_before: f64,
    #[serde(default)]
    pub equity_with_loan: f64,
    #[serde(default)]
    pub equity_with_loan_before: f64,
    #[serde(default, rename = "availableEE")]
    pub available_ee: f64,
    #[serde(default)]
    pub excess_liquidity: f64,
    #[serde(default)]
    pub overnight_liquidation: f64,
    #[serde(default)]
    pub gst: f64,
    #[serde(default)]
    pub message: String,
}

/// 下单返回结果。`id` 是全局订单 ID，`order_id` 是账户自增号。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PlaceOrderResult {
    #[serde(default)]
    pub id: i64,
    /// 账户自增订单号。服务端字段名为 `order_id`（snake_case）。
    #[serde(default, rename = "order_id")]
    pub order_id: i64,
    #[serde(default, rename = "subIds")]
    pub sub_ids: Vec<i64>,
    #[serde(default)]
    pub orders: Vec<crate::model::order::Order>,
}

/// 仅含订单 ID 的响应（ModifyOrder/CancelOrder）
#[derive(Debug, Clone, Deserialize, Default)]
pub struct OrderIdResult {
    #[serde(default)]
    pub id: i64,
}

/// 成交记录
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub order_id: i64,
    /// 账户 ID（数字型）
    #[serde(default)]
    pub account_id: i64,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub sec_type: String,
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub identifier: String,
    #[serde(default)]
    pub action: String,
    /// 委托价
    #[serde(default)]
    pub price: f64,
    /// 成交价（服务端字段名 filledPrice）
    #[serde(default)]
    pub filled_price: f64,
    #[serde(default)]
    pub quantity: i64,
    #[serde(default)]
    pub filled_quantity: i64,
    #[serde(default)]
    pub filled_quantity_scale: i32,
    /// 委托金额
    #[serde(default)]
    pub amount: f64,
    /// 成交金额（服务端字段名 filledAmount）
    #[serde(default)]
    pub filled_amount: f64,
    #[serde(default)]
    pub commission: f64,
    /// 成交时间字符串，格式 "YYYY-MM-DD HH:MM:SS"（服务端返回字符串，非时间戳）
    #[serde(default)]
    pub transacted_at: String,
    /// 成交时间毫秒时间戳
    #[serde(default)]
    pub transaction_time: i64,
    /// 兼容旧字段
    #[serde(default)]
    pub time: i64,
}

/// 机构子账户信息（来自 /accounts）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ManagedAccount {
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub account_type: String,
    #[serde(default)]
    pub capability: String,
    #[serde(default)]
    pub status: String,
}

/// 资产分析（按日）条目。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsAsset {
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub holding_value: f64,
    #[serde(default)]
    pub cash_balance: f64,
    #[serde(default)]
    pub pnl: f64,
    #[serde(default)]
    pub pnl_rate: f64,
    #[serde(default)]
    pub net_value_index: f64,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub seg_type: String,
}

/// 综合账户总览。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AggregateAssets {
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub net_liquidation: f64,
    #[serde(default)]
    pub gross_position_value: f64,
    #[serde(default)]
    pub cash_balance: f64,
    #[serde(default)]
    pub base_currency: String,
    #[serde(default)]
    pub currency_assets: Vec<CurrencyAsset>,
}

/// 可交易数量估算结果。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EstimateTradableQuantity {
    #[serde(default)]
    pub tradable_quantity: f64,
    #[serde(default)]
    pub max_cash_buy_quantity: f64,
    #[serde(default)]
    pub max_margin_buy_quantity: f64,
    #[serde(default)]
    pub max_short_sell_quantity: f64,
    #[serde(default)]
    pub max_position_sell_quantity: f64,
    #[serde(default)]
    pub cash_buying_power: f64,
    #[serde(default)]
    pub currency: String,
}

/// 外汇下单返回结果。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ForexOrderResult {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub source_currency: String,
    #[serde(default)]
    pub target_currency: String,
    #[serde(default)]
    pub source_amount: f64,
    #[serde(default)]
    pub target_amount: f64,
    #[serde(default)]
    pub rate: f64,
    #[serde(default)]
    pub submit_time: i64,
}

/// 子账户资金调拨响应（transfer_segment_fund / cancel_segment_fund 共用）。
///
/// `id` 使用 [`serde_json::Value`] 因为服务端可能返回数字或字符串（与 Go SDK 保持一致）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SegmentFund {
    /// 调拨单 ID（服务端可能返回数字或字符串）
    #[serde(default)]
    pub id: Value,
    #[serde(default)]
    pub from_segment: String,
    #[serde(default)]
    pub to_segment: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub status_desc: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub settled_at: i64,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

/// 可调拨资金条目（segment_fund_available 响应）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SegmentFundAvailableItem {
    #[serde(default)]
    pub from_segment: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub amount: f64,
}

/// 资金调拨历史条目。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SegmentFundHistoryItem {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub from_segment: String,
    #[serde(default)]
    pub to_segment: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub status_desc: String,
    #[serde(default)]
    pub settled_at: i64,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

/// 资金明细条目（/fund_details）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FundDetails {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub seg_type: String,
    #[serde(default)]
    pub fund_type: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub balance: f64,
    #[serde(default)]
    pub occur_time: i64,
    #[serde(default)]
    pub remark: String,
    #[serde(default)]
    pub external_id: String,
}

/// 调拨记录（/transfer_fund）。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FundingHistoryItem {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub ref_id: String,
    /// 资金类型编码
    #[serde(default, rename = "type")]
    pub fund_type: i32,
    #[serde(default)]
    pub type_desc: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub business_date: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub status_desc: String,
    #[serde(default)]
    pub completed_status: bool,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

/// 内部转股单项（响应结构，含 camelCase 反序列化）。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TransferItemResponse {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub quantity: i64,
    #[serde(default)]
    pub expiry: String,
    #[serde(default)]
    pub strike: String,
    #[serde(default)]
    pub right: String,
    #[serde(default)]
    pub sec_type: String,
}

/// 内部转股记录。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PositionTransferRecord {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub from_account: String,
    #[serde(default)]
    pub to_account: String,
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub submit_time: i64,
    #[serde(default)]
    pub transfers: Option<Vec<TransferItemResponse>>,
}

/// 内部转股详情。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PositionTransferDetail {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub from_account: String,
    #[serde(default)]
    pub to_account: String,
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub submit_time: i64,
    #[serde(default)]
    pub update_time: i64,
    #[serde(default)]
    pub transfers: Vec<TransferItemResponse>,
    #[serde(default)]
    pub remark: String,
}

/// 外部转股记录。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PositionTransferExternalRecord {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub quantity: i64,
    #[serde(default)]
    pub direction: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub submit_time: i64,
    #[serde(default)]
    pub update_time: i64,
}

/// 行权检验结果
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionExerciseCheckResult {
    #[serde(default)]
    pub available_quantity: f64,
    #[serde(default)]
    pub position: f64,
    #[serde(default)]
    pub stk_position: f64,
    #[serde(default)]
    pub stk_position_change: f64,
    #[serde(default)]
    pub stk_position_before: f64,
    #[serde(default)]
    pub stk_position_after: f64,
    #[serde(default)]
    pub symbol: String,
}

/// 可行权期权持仓条目
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionExercisePosition {
    #[serde(default)]
    pub contract_id: i64,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub stk_symbol: String,
    #[serde(default)]
    pub expire_date: String,
    #[serde(default)]
    pub strike: String,
    #[serde(default)]
    pub call_put: String,
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub account_id: i64,
    #[serde(default)]
    pub position: f64,
    #[serde(default)]
    pub available_quantity: f64,
}

/// 可行权持仓分页结果
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionExercisePositionPageResult {
    #[serde(default)]
    pub page_num: i32,
    #[serde(default)]
    pub page_size: i32,
    #[serde(default)]
    pub item_count: i32,
    #[serde(default)]
    pub page_count: i32,
    #[serde(default)]
    pub items: Vec<OptionExercisePosition>,
}

/// 行权申请记录条目
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionExerciseRecord {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub contract_id: i64,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub stk_symbol: String,
    #[serde(default)]
    pub expire_date: String,
    #[serde(default)]
    pub strike: String,
    #[serde(default)]
    pub call_put: String,
    /// Exercise | Expire
    #[serde(default, rename = "type")]
    pub exercise_type: String,
    #[serde(default)]
    pub request_quantity: f64,
    #[serde(default)]
    pub quantity: f64,
    /// New | Cancel | Success | Fail
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub executing_date: String,
    #[serde(default)]
    pub itm_rate: i32,
    #[serde(default)]
    pub is_force: bool,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub account_id: i64,
}

/// 行权记录分页结果
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionExerciseRecordPageResult {
    #[serde(default)]
    pub page_num: i32,
    #[serde(default)]
    pub page_size: i32,
    #[serde(default)]
    pub item_count: i32,
    #[serde(default)]
    pub page_count: i32,
    #[serde(default)]
    pub items: Vec<OptionExerciseRecord>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_deserialize() {
        let json = r#"{"account":"DU123","buyingPower":100000.0,"netLiquidation":200000.0,"currency":"USD","segments":[{"category":"S","buyingPower":50000.0,"availableFunds":40000.0}]}"#;
        let a: Asset = serde_json::from_str(json).unwrap();
        assert_eq!(a.account, "DU123");
        assert_eq!(a.buying_power, 100000.0);
        assert_eq!(a.net_liquidation, 200000.0);
        assert_eq!(a.segments.len(), 1);
    }

    #[test]
    fn test_prime_asset_deserialize() {
        let json = r#"{"accountId":"acc1","updateTimestamp":1700000000,"segments":[{"capability":"MARGIN","category":"S","currency":"USD","buyingPower":10000.0,"netLiquidation":20000.0}]}"#;
        let p: PrimeAsset = serde_json::from_str(json).unwrap();
        assert_eq!(p.account_id, "acc1");
        assert_eq!(p.segments.len(), 1);
        assert_eq!(p.segments[0].buying_power, 10000.0);
    }

    #[test]
    fn test_preview_result_deserialize() {
        let json = r#"{"account":"DU123","isPass":true,"commission":0.5,"commissionCurrency":"USD","initMargin":50.0,"maintMargin":25.0}"#;
        let p: PreviewResult = serde_json::from_str(json).unwrap();
        assert!(p.is_pass);
        assert_eq!(p.commission, 0.5);
        assert_eq!(p.init_margin, 50.0);
    }

    #[test]
    fn test_place_order_result_deserialize() {
        let json = r#"{"id":42519413060422656,"order_id":143}"#;
        let r: PlaceOrderResult = serde_json::from_str(json).unwrap();
        assert_eq!(r.id, 42519413060422656);
        assert_eq!(r.order_id, 143);
    }

    #[test]
    fn test_order_id_result_deserialize() {
        let json = r#"{"id":12345}"#;
        let r: OrderIdResult = serde_json::from_str(json).unwrap();
        assert_eq!(r.id, 12345);
    }

    #[test]
    fn test_transaction_deserialize() {
        let json = r#"{"id":1,"orderId":2,"accountId":99,"symbol":"AAPL","secType":"STK","price":150.0,"filledPrice":149.5,"quantity":100,"filledQuantity":100,"filledQuantityScale":0,"amount":15000.0,"filledAmount":14950.0,"transactedAt":"2024-01-15 10:30:00","transactionTime":1705314600000}"#;
        let t: Transaction = serde_json::from_str(json).unwrap();
        assert_eq!(t.id, 1);
        assert_eq!(t.order_id, 2);
        assert_eq!(t.account_id, 99);
        assert_eq!(t.sec_type, "STK");
        assert_eq!(t.price, 150.0);
        assert_eq!(t.filled_price, 149.5);
        assert_eq!(t.filled_amount, 14950.0);
        assert_eq!(t.transacted_at, "2024-01-15 10:30:00");
        assert_eq!(t.transaction_time, 1705314600000);
    }
}
