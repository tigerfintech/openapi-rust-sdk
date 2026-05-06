//! 交易响应模型：Asset、PrimeAsset、PreviewResult、PlaceOrderResult、OrderIdResult、Transaction。

use serde::Deserialize;

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
    #[serde(default)]
    pub price: f64,
    #[serde(default)]
    pub quantity: i64,
    #[serde(default)]
    pub filled_quantity: i64,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub commission: f64,
    #[serde(default)]
    pub transacted_at: i64,
    #[serde(default)]
    pub time: i64,
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
        let json = r#"{"id":1,"orderId":2,"symbol":"AAPL","secType":"STK","price":150.0,"quantity":100,"filledQuantity":100,"transactedAt":1700000000}"#;
        let t: Transaction = serde_json::from_str(json).unwrap();
        assert_eq!(t.id, 1);
        assert_eq!(t.order_id, 2);
        assert_eq!(t.sec_type, "STK");
        assert_eq!(t.price, 150.0);
    }
}
