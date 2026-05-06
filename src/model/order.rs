//! 订单模型定义和构造工具函数。
//!
//! - `Order`：查询类接口返回的订单数据，使用 `#[serde(rename_all = "camelCase")]`。
//! - `OrderRequest`：下单/改单/预览订单接口的请求体，使用 `#[serde(rename_all = "snake_case")]`。

use serde::{Deserialize, Serialize};

// ========== 响应模型（查询类接口返回） ==========

/// 附加订单（止盈/止损）- 响应模型
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderLeg {
    #[serde(default)]
    pub leg_type: String,
    #[serde(default)]
    pub price: f64,
    #[serde(default)]
    pub time_in_force: String,
    #[serde(default)]
    pub quantity: i64,
}

/// 算法订单参数 - 响应模型
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AlgoParams {
    #[serde(default)]
    pub algo_strategy: String,
    #[serde(default)]
    pub start_time: String,
    #[serde(default)]
    pub end_time: String,
    #[serde(default)]
    pub participation_rate: f64,
}

/// 订单响应模型。
///
/// 服务端响应字段为 camelCase。下单/改单请使用 [`OrderRequest`]。
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    #[serde(default)]
    pub account: String,
    /// 全局订单 ID
    #[serde(default)]
    pub id: i64,
    /// 账户自增订单号
    #[serde(default)]
    pub order_id: i64,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub order_type: String,
    #[serde(default)]
    pub total_quantity: i64,
    #[serde(default)]
    pub limit_price: f64,
    #[serde(default)]
    pub aux_price: f64,
    #[serde(default)]
    pub trailing_percent: f64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub filled_quantity: i64,
    #[serde(default)]
    pub avg_fill_price: f64,
    #[serde(default)]
    pub time_in_force: String,
    #[serde(default)]
    pub outside_rth: bool,
    #[serde(default)]
    pub order_legs: Vec<OrderLeg>,
    #[serde(default)]
    pub algo_params: Option<AlgoParams>,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub sec_type: String,
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub expiry: String,
    #[serde(default)]
    pub strike: String,
    #[serde(default)]
    pub right: String,
    #[serde(default)]
    pub identifier: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub commission: f64,
    #[serde(default)]
    pub realized_pnl: f64,
    #[serde(default)]
    pub open_time: i64,
    #[serde(default)]
    pub update_time: i64,
    #[serde(default)]
    pub latest_time: i64,
    #[serde(default)]
    pub remark: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub user_mark: String,
    #[serde(default)]
    pub external_id: String,
    #[serde(default)]
    pub total_quantity_scale: i32,
    #[serde(default)]
    pub filled_quantity_scale: i32,
    #[serde(default)]
    pub filled_cash_amount: f64,
    #[serde(default)]
    pub gst: f64,
    #[serde(default)]
    pub liquidation: bool,
    #[serde(default)]
    pub attr_desc: String,
    #[serde(default)]
    pub attr_list: Vec<String>,
    #[serde(default)]
    pub algo_strategy: String,
    #[serde(default)]
    pub discount: f64,
    #[serde(default)]
    pub replace_status: String,
    #[serde(default)]
    pub cancel_status: String,
    #[serde(default)]
    pub can_modify: bool,
    #[serde(default)]
    pub can_cancel: bool,
    #[serde(default)]
    pub is_open: bool,
    #[serde(default)]
    pub order_discount: f64,
    #[serde(default)]
    pub trading_session_type: String,
    #[serde(default)]
    pub latest_price: f64,
}

// ========== 请求模型（下单/改单/预览） ==========

/// 附加订单（止盈/止损）- 请求模型
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct OrderLegRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leg_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,
}

/// 算法订单参数 - 请求模型
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AlgoParamsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algo_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participation_rate: Option<f64>,
}

/// 订单请求模型。
///
/// 服务端请求体字段为 snake_case。此结构体用于 `place_order` / `preview_order`
/// / `modify_order` 接口。查询返回请使用 [`Order`]。
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct OrderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// 全局订单 ID（修改订单时必填）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_quantity: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aux_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outside_rth: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_legs: Option<Vec<OrderLegRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algo_params: Option<AlgoParamsRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
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
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_mark: Option<String>,
}

// ========== 订单请求构造工具函数 ==========

/// 构造市价单
pub fn market_order(account: &str, symbol: &str, sec_type: &str, action: &str, quantity: i64) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("MKT".to_string()),
        total_quantity: Some(quantity),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造限价单
pub fn limit_order(account: &str, symbol: &str, sec_type: &str, action: &str, quantity: i64, limit_price: f64) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("LMT".to_string()),
        total_quantity: Some(quantity),
        limit_price: Some(limit_price),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造止损单
pub fn stop_order(account: &str, symbol: &str, sec_type: &str, action: &str, quantity: i64, aux_price: f64) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("STP".to_string()),
        total_quantity: Some(quantity),
        aux_price: Some(aux_price),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造止损限价单
pub fn stop_limit_order(
    account: &str, symbol: &str, sec_type: &str, action: &str,
    quantity: i64, limit_price: f64, aux_price: f64,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("STP_LMT".to_string()),
        total_quantity: Some(quantity),
        limit_price: Some(limit_price),
        aux_price: Some(aux_price),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造跟踪止损单
pub fn trail_order(
    account: &str, symbol: &str, sec_type: &str, action: &str,
    quantity: i64, trailing_percent: f64,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("TRAIL".to_string()),
        total_quantity: Some(quantity),
        trailing_percent: Some(trailing_percent),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造竞价限价单
pub fn auction_limit_order(
    account: &str, symbol: &str, sec_type: &str, action: &str,
    quantity: i64, limit_price: f64,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("AL".to_string()),
        total_quantity: Some(quantity),
        limit_price: Some(limit_price),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造竞价市价单
pub fn auction_market_order(account: &str, symbol: &str, sec_type: &str, action: &str, quantity: i64) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("AM".to_string()),
        total_quantity: Some(quantity),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造算法订单（TWAP/VWAP）
pub fn algo_order(
    account: &str, symbol: &str, sec_type: &str, action: &str,
    quantity: i64, limit_price: f64, algo_type: &str, params: AlgoParamsRequest,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some(algo_type.to_string()),
        total_quantity: Some(quantity),
        limit_price: Some(limit_price),
        algo_params: Some(params),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造附加订单（止盈/止损）
pub fn new_order_leg(leg_type: &str, price: f64, time_in_force: &str) -> OrderLegRequest {
    OrderLegRequest {
        leg_type: Some(leg_type.to_string()),
        price: Some(price),
        time_in_force: Some(time_in_force.to_string()),
        quantity: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_request_serializes_to_snake_case() {
        let order = limit_order("ACC123", "AAPL", "STK", "BUY", 100, 150.50);
        let json_value: serde_json::Value = serde_json::to_value(&order).unwrap();
        let obj = json_value.as_object().unwrap();

        // snake_case 请求体
        assert!(obj.contains_key("sec_type"), "request should use sec_type");
        assert!(obj.contains_key("order_type"), "request should use order_type");
        assert!(obj.contains_key("total_quantity"), "request should use total_quantity");
        assert!(obj.contains_key("limit_price"), "request should use limit_price");
        assert!(obj.contains_key("time_in_force"), "request should use time_in_force");

        // 不应出现 camelCase
        assert!(!obj.contains_key("secType"));
        assert!(!obj.contains_key("orderType"));
        assert!(!obj.contains_key("totalQuantity"));
    }

    #[test]
    fn test_order_response_parses_camel_case() {
        let json = r#"{
            "account": "ACC123",
            "id": 42519413060422656,
            "orderId": 143,
            "action": "BUY",
            "orderType": "MKT",
            "totalQuantity": 100,
            "filledQuantity": 100,
            "avgFillPrice": 543.5,
            "timeInForce": "DAY",
            "outsideRth": false,
            "symbol": "00700",
            "secType": "STK",
            "market": "HK",
            "status": "Filled",
            "commission": 92.38,
            "realizedPnl": 0.0,
            "name": "TENCENT",
            "identifier": "00700",
            "source": "openapi",
            "userMark": "test001",
            "openTime": 1773296577000,
            "updateTime": 1773590598000
        }"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.account, "ACC123");
        assert_eq!(order.id, 42519413060422656);
        assert_eq!(order.order_id, 143);
        assert_eq!(order.total_quantity, 100);
        assert_eq!(order.filled_quantity, 100);
        assert_eq!(order.avg_fill_price, 543.5);
        assert_eq!(order.status, "Filled");
        assert_eq!(order.sec_type, "STK");
        assert_eq!(order.name, "TENCENT");
        assert_eq!(order.user_mark, "test001");
        assert_eq!(order.open_time, 1773296577000);
    }

    #[test]
    fn test_market_order_helper() {
        let o = market_order("ACC", "AAPL", "STK", "BUY", 100);
        assert_eq!(o.total_quantity, Some(100));
        assert_eq!(o.order_type, Some("MKT".to_string()));
    }

    #[test]
    fn test_limit_order_helper() {
        let o = limit_order("ACC", "AAPL", "STK", "SELL", 50, 155.0);
        assert_eq!(o.order_type, Some("LMT".to_string()));
        assert_eq!(o.limit_price, Some(155.0));
    }

    #[test]
    fn test_stop_order_helper() {
        let o = stop_order("ACC", "AAPL", "STK", "SELL", 100, 140.0);
        assert_eq!(o.order_type, Some("STP".to_string()));
        assert_eq!(o.aux_price, Some(140.0));
    }

    #[test]
    fn test_new_order_leg_helper() {
        let leg = new_order_leg("PROFIT", 160.0, "GTC");
        assert_eq!(leg.leg_type, Some("PROFIT".to_string()));
        assert_eq!(leg.price, Some(160.0));
    }

    #[test]
    fn test_order_request_skip_none_fields() {
        let order = market_order("ACC", "AAPL", "STK", "BUY", 100);
        let json_value: serde_json::Value = serde_json::to_value(&order).unwrap();
        let obj = json_value.as_object().unwrap();

        // 必填字段存在
        assert!(obj.contains_key("symbol"));
        assert!(obj.contains_key("sec_type"));
        assert!(obj.contains_key("order_type"));
        assert!(obj.contains_key("total_quantity"));

        // None 字段不应出现
        assert!(!obj.contains_key("id"));
        assert!(!obj.contains_key("limit_price"));
        assert!(!obj.contains_key("aux_price"));
    }
}
