//! 订单模型定义和构造工具函数。
//! 字段名词根保持与 API JSON 一致，通过 serde rename 映射到 API JSON 字段名。

use serde::{Deserialize, Serialize};

/// 附加订单（止盈/止损）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderLeg {
    /// 附加订单类型（PROFIT/LOSS）
    #[serde(rename = "legType", skip_serializing_if = "Option::is_none")]
    pub leg_type: Option<String>,
    /// 价格
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    /// 有效期
    #[serde(rename = "timeInForce", skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    /// 数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,
}

/// 算法订单参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlgoParams {
    /// 算法策略（TWAP/VWAP）
    #[serde(rename = "algoStrategy", skip_serializing_if = "Option::is_none")]
    pub algo_strategy: Option<String>,
    /// 开始时间
    #[serde(rename = "startTime", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    /// 结束时间
    #[serde(rename = "endTime", skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    /// 参与率
    #[serde(rename = "participationRate", skip_serializing_if = "Option::is_none")]
    pub participation_rate: Option<f64>,
}

/// 订单模型，字段名词根保持与 API JSON 一致。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
    /// 交易账户
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// 全局订单 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// 账户自增订单号
    #[serde(rename = "orderId", skip_serializing_if = "Option::is_none")]
    pub order_id: Option<i64>,
    /// 买卖方向（BUY/SELL）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// 订单类型（MKT/LMT/STP/STP_LMT/TRAIL 等）
    #[serde(rename = "orderType", skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    /// 总数量（API 返回字段名为 totalQuantity）
    #[serde(rename = "totalQuantity", skip_serializing_if = "Option::is_none")]
    pub total_quantity: Option<i64>,
    /// 限价
    #[serde(rename = "limitPrice", skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<f64>,
    /// 辅助价格（止损价）
    #[serde(rename = "auxPrice", skip_serializing_if = "Option::is_none")]
    pub aux_price: Option<f64>,
    /// 跟踪止损百分比
    #[serde(rename = "trailingPercent", skip_serializing_if = "Option::is_none")]
    pub trailing_percent: Option<f64>,
    /// 订单状态
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// 已成交数量（API 返回字段名为 filledQuantity）
    #[serde(rename = "filledQuantity", skip_serializing_if = "Option::is_none")]
    pub filled_quantity: Option<i64>,
    /// 平均成交价
    #[serde(rename = "avgFillPrice", skip_serializing_if = "Option::is_none")]
    pub avg_fill_price: Option<f64>,
    /// 有效期（DAY/GTC/OPG）
    #[serde(rename = "timeInForce", skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    /// 是否允许盘前盘后
    #[serde(rename = "outsideRth", skip_serializing_if = "Option::is_none")]
    pub outside_rth: Option<bool>,
    /// 附加订单（止盈/止损）
    #[serde(rename = "orderLegs", skip_serializing_if = "Option::is_none")]
    pub order_legs: Option<Vec<OrderLeg>>,
    /// 算法参数
    #[serde(rename = "algoParams", skip_serializing_if = "Option::is_none")]
    pub algo_params: Option<AlgoParams>,
    /// 股票代码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// 合约类型
    #[serde(rename = "secType", skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    /// 市场
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    /// 货币
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// 到期日（期权/期货）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    /// 行权价（期权），API 返回为字符串
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike: Option<String>,
    /// 看涨/看跌（PUT/CALL），保持 API 原始名 right
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    /// 合约标识符
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    /// 合约名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 佣金
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission: Option<f64>,
    /// 已实现盈亏
    #[serde(rename = "realizedPnl", skip_serializing_if = "Option::is_none")]
    pub realized_pnl: Option<f64>,
    /// 开仓时间（毫秒时间戳）
    #[serde(rename = "openTime", skip_serializing_if = "Option::is_none")]
    pub open_time: Option<i64>,
    /// 更新时间（毫秒时间戳）
    #[serde(rename = "updateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<i64>,
    /// 最新时间（毫秒时间戳）
    #[serde(rename = "latestTime", skip_serializing_if = "Option::is_none")]
    pub latest_time: Option<i64>,
    /// 备注
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    /// 订单来源
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// 用户标记
    #[serde(rename = "userMark", skip_serializing_if = "Option::is_none")]
    pub user_mark: Option<String>,
}

/// 构造市价单
pub fn market_order(account: &str, symbol: &str, sec_type: &str, action: &str, quantity: i64) -> Order {
    Order {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("MKT".to_string()),
        total_quantity: Some(quantity),
        time_in_force: Some("DAY".to_string()),
        ..Order::default()
    }
}

/// 构造限价单
pub fn limit_order(account: &str, symbol: &str, sec_type: &str, action: &str, quantity: i64, limit_price: f64) -> Order {
    Order {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("LMT".to_string()),
        total_quantity: Some(quantity),
        limit_price: Some(limit_price),
        time_in_force: Some("DAY".to_string()),
        ..Order::default()
    }
}

/// 构造止损单
pub fn stop_order(account: &str, symbol: &str, sec_type: &str, action: &str, quantity: i64, aux_price: f64) -> Order {
    Order {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("STP".to_string()),
        total_quantity: Some(quantity),
        aux_price: Some(aux_price),
        time_in_force: Some("DAY".to_string()),
        ..Order::default()
    }
}

/// 构造止损限价单
pub fn stop_limit_order(
    account: &str, symbol: &str, sec_type: &str, action: &str,
    quantity: i64, limit_price: f64, aux_price: f64,
) -> Order {
    Order {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("STP_LMT".to_string()),
        total_quantity: Some(quantity),
        limit_price: Some(limit_price),
        aux_price: Some(aux_price),
        time_in_force: Some("DAY".to_string()),
        ..Order::default()
    }
}

/// 构造跟踪止损单
pub fn trail_order(
    account: &str, symbol: &str, sec_type: &str, action: &str,
    quantity: i64, trailing_percent: f64,
) -> Order {
    Order {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("TRAIL".to_string()),
        total_quantity: Some(quantity),
        trailing_percent: Some(trailing_percent),
        time_in_force: Some("DAY".to_string()),
        ..Order::default()
    }
}

/// 构造竞价限价单
pub fn auction_limit_order(
    account: &str, symbol: &str, sec_type: &str, action: &str,
    quantity: i64, limit_price: f64,
) -> Order {
    Order {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("AL".to_string()),
        total_quantity: Some(quantity),
        limit_price: Some(limit_price),
        time_in_force: Some("DAY".to_string()),
        ..Order::default()
    }
}

/// 构造竞价市价单
pub fn auction_market_order(account: &str, symbol: &str, sec_type: &str, action: &str, quantity: i64) -> Order {
    Order {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("AM".to_string()),
        total_quantity: Some(quantity),
        time_in_force: Some("DAY".to_string()),
        ..Order::default()
    }
}

/// 构造算法订单（TWAP/VWAP）
pub fn algo_order(
    account: &str, symbol: &str, sec_type: &str, action: &str,
    quantity: i64, limit_price: f64, algo_type: &str, params: AlgoParams,
) -> Order {
    Order {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some(algo_type.to_string()),
        total_quantity: Some(quantity),
        limit_price: Some(limit_price),
        algo_params: Some(params),
        time_in_force: Some("DAY".to_string()),
        ..Order::default()
    }
}

/// 构造附加订单（止盈/止损）
pub fn new_order_leg(leg_type: &str, price: f64, time_in_force: &str) -> OrderLeg {
    OrderLeg {
        leg_type: Some(leg_type.to_string()),
        price: Some(price),
        time_in_force: Some(time_in_force.to_string()),
        quantity: None,
    }
}

impl Default for Order {
    fn default() -> Self {
        Order {
            account: None,
            id: None,
            order_id: None,
            action: None,
            order_type: None,
            total_quantity: None,
            limit_price: None,
            aux_price: None,
            trailing_percent: None,
            status: None,
            filled_quantity: None,
            avg_fill_price: None,
            time_in_force: None,
            outside_rth: None,
            order_legs: None,
            algo_params: None,
            symbol: None,
            sec_type: None,
            market: None,
            currency: None,
            expiry: None,
            strike: None,
            right: None,
            identifier: None,
            name: None,
            commission: None,
            realized_pnl: None,
            open_time: None,
            update_time: None,
            latest_time: None,
            remark: None,
            source: None,
            user_mark: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_json_round_trip_market_order() {
        let order = market_order("ACC123", "AAPL", "STK", "BUY", 100);
        let json = serde_json::to_string(&order).unwrap();
        let deserialized: Order = serde_json::from_str(&json).unwrap();
        assert_eq!(order, deserialized);
    }

    #[test]
    fn test_order_json_round_trip_limit_order() {
        let order = limit_order("ACC123", "AAPL", "STK", "BUY", 100, 150.50);
        let json = serde_json::to_string(&order).unwrap();
        let deserialized: Order = serde_json::from_str(&json).unwrap();
        assert_eq!(order, deserialized);
    }

    /// 验证 serde rename 映射到 API JSON 字段名
    #[test]
    fn test_order_serde_rename_field_names() {
        let order = Order {
            account: Some("ACC123".to_string()),
            id: Some(1),
            order_id: Some(100),
            action: Some("BUY".to_string()),
            order_type: Some("LMT".to_string()),
            total_quantity: Some(100),
            limit_price: Some(150.0),
            status: Some("Filled".to_string()),
            filled_quantity: Some(100),
            avg_fill_price: Some(149.5),
            time_in_force: Some("DAY".to_string()),
            outside_rth: Some(true),
            symbol: Some("AAPL".to_string()),
            sec_type: Some("STK".to_string()),
            ..Order::default()
        };
        let json_value: serde_json::Value = serde_json::to_value(&order).unwrap();
        let obj = json_value.as_object().unwrap();

        assert!(obj.contains_key("orderId"), "应映射为 orderId");
        assert!(obj.contains_key("orderType"), "应映射为 orderType");
        assert!(obj.contains_key("totalQuantity"), "应映射为 totalQuantity");
        assert!(obj.contains_key("filledQuantity"), "应映射为 filledQuantity");
        assert!(obj.contains_key("limitPrice"), "应映射为 limitPrice");
        assert!(obj.contains_key("avgFillPrice"), "应映射为 avgFillPrice");
        assert!(obj.contains_key("timeInForce"), "应映射为 timeInForce");
        assert!(obj.contains_key("outsideRth"), "应映射为 outsideRth");
        assert!(obj.contains_key("secType"), "应映射为 secType");

        // 验证不存在 snake_case 字段名
        assert!(!obj.contains_key("order_id"));
        assert!(!obj.contains_key("order_type"));
        assert!(!obj.contains_key("total_quantity"));
        assert!(!obj.contains_key("filled_quantity"));
    }

    /// 验证从 API JSON 反序列化（使用真实 API 字段名）
    #[test]
    fn test_order_deserialize_from_api_json() {
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
        assert_eq!(order.account, Some("ACC123".to_string()));
        assert_eq!(order.id, Some(42519413060422656));
        assert_eq!(order.order_id, Some(143));
        assert_eq!(order.total_quantity, Some(100));
        assert_eq!(order.filled_quantity, Some(100));
        assert_eq!(order.avg_fill_price, Some(543.5));
        assert_eq!(order.status, Some("Filled".to_string()));
        assert_eq!(order.commission, Some(92.38));
        assert_eq!(order.name, Some("TENCENT".to_string()));
        assert_eq!(order.source, Some("openapi".to_string()));
        assert_eq!(order.user_mark, Some("test001".to_string()));
        assert_eq!(order.open_time, Some(1773296577000));
    }

    #[test]
    fn test_market_order() {
        let o = market_order("ACC", "AAPL", "STK", "BUY", 100);
        assert_eq!(o.total_quantity, Some(100));
        assert_eq!(o.order_type, Some("MKT".to_string()));
    }

    #[test]
    fn test_limit_order() {
        let o = limit_order("ACC", "AAPL", "STK", "SELL", 50, 155.0);
        assert_eq!(o.order_type, Some("LMT".to_string()));
        assert_eq!(o.limit_price, Some(155.0));
    }

    #[test]
    fn test_stop_order() {
        let o = stop_order("ACC", "AAPL", "STK", "SELL", 100, 140.0);
        assert_eq!(o.order_type, Some("STP".to_string()));
        assert_eq!(o.aux_price, Some(140.0));
    }

    #[test]
    fn test_new_order_leg() {
        let leg = new_order_leg("PROFIT", 160.0, "GTC");
        assert_eq!(leg.leg_type, Some("PROFIT".to_string()));
        assert_eq!(leg.price, Some(160.0));
    }
}
