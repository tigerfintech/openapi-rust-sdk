//! 持仓模型定义。
//! 字段名词根保持与 API JSON 一致，通过 serde rename 映射到 API JSON 字段名。

use serde::{Deserialize, Serialize};

/// 持仓模型，字段名词根保持与 API JSON 一致。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    /// 交易账户
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// 标的代码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// 证券类型
    #[serde(rename = "secType", skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    /// 市场
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    /// 货币
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// 持仓数量（API 返回字段名为 position）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// 平均成本
    #[serde(rename = "averageCost", skip_serializing_if = "Option::is_none")]
    pub average_cost: Option<f64>,
    /// 市值
    #[serde(rename = "marketValue", skip_serializing_if = "Option::is_none")]
    pub market_value: Option<f64>,
    /// 已实现盈亏
    #[serde(rename = "realizedPnl", skip_serializing_if = "Option::is_none")]
    pub realized_pnl: Option<f64>,
    /// 未实现盈亏
    #[serde(rename = "unrealizedPnl", skip_serializing_if = "Option::is_none")]
    pub unrealized_pnl: Option<f64>,
    /// 未实现盈亏百分比
    #[serde(rename = "unrealizedPnlPercent", skip_serializing_if = "Option::is_none")]
    pub unrealized_pnl_percent: Option<f64>,
    /// 合约 ID
    #[serde(rename = "contractId", skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<i64>,
    /// 合约标识符
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    /// 合约名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 最新价格
    #[serde(rename = "latestPrice", skip_serializing_if = "Option::is_none")]
    pub latest_price: Option<f64>,
    /// 合约乘数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiplier: Option<f64>,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            account: None,
            symbol: None,
            sec_type: None,
            market: None,
            currency: None,
            position: None,
            average_cost: None,
            market_value: None,
            realized_pnl: None,
            unrealized_pnl: None,
            unrealized_pnl_percent: None,
            contract_id: None,
            identifier: None,
            name: None,
            latest_price: None,
            multiplier: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_json_round_trip() {
        let position = Position {
            account: Some("ACC123".to_string()),
            symbol: Some("AAPL".to_string()),
            sec_type: Some("STK".to_string()),
            market: Some("US".to_string()),
            currency: Some("USD".to_string()),
            position: Some(100),
            average_cost: Some(150.25),
            market_value: Some(15500.0),
            realized_pnl: Some(500.0),
            unrealized_pnl: Some(475.0),
            ..Position::default()
        };
        let json = serde_json::to_string(&position).unwrap();
        let deserialized: Position = serde_json::from_str(&json).unwrap();
        assert_eq!(position, deserialized);
    }

    /// 验证 serde rename 映射到 API JSON 字段名
    #[test]
    fn test_position_serde_rename_field_names() {
        let position = Position {
            account: Some("ACC123".to_string()),
            symbol: Some("AAPL".to_string()),
            sec_type: Some("STK".to_string()),
            position: Some(100),
            average_cost: Some(150.25),
            market_value: Some(15500.0),
            unrealized_pnl: Some(475.0),
            unrealized_pnl_percent: Some(-0.05),
            contract_id: Some(14),
            latest_price: Some(155.0),
            ..Position::default()
        };
        let json_value: serde_json::Value = serde_json::to_value(&position).unwrap();
        let obj = json_value.as_object().unwrap();

        assert!(obj.contains_key("secType"), "应映射为 secType");
        assert!(obj.contains_key("averageCost"), "应映射为 averageCost");
        assert!(obj.contains_key("marketValue"), "应映射为 marketValue");
        assert!(obj.contains_key("unrealizedPnl"), "应映射为 unrealizedPnl");
        assert!(obj.contains_key("unrealizedPnlPercent"), "应映射为 unrealizedPnlPercent");
        assert!(obj.contains_key("contractId"), "应映射为 contractId");
        assert!(obj.contains_key("latestPrice"), "应映射为 latestPrice");
        assert!(obj.contains_key("position"), "应包含 position 字段");

        // 验证不存在 snake_case 字段名
        assert!(!obj.contains_key("sec_type"));
        assert!(!obj.contains_key("average_cost"));
        assert!(!obj.contains_key("market_value"));
        assert!(!obj.contains_key("unrealized_pnl"));
        assert!(!obj.contains_key("contract_id"));
        assert!(!obj.contains_key("latest_price"));
    }

    /// 验证从 API JSON 反序列化（使用真实 API 字段名）
    #[test]
    fn test_position_deserialize_from_api_json() {
        let json = r#"{
            "account": "DU123456",
            "symbol": "00700",
            "secType": "STK",
            "market": "HK",
            "currency": "HKD",
            "position": 11600,
            "averageCost": 631.6747,
            "marketValue": 5614400.0,
            "realizedPnl": -11161.9,
            "unrealizedPnl": -1713026.17,
            "unrealizedPnlPercent": -0.2338,
            "contractId": 19083,
            "identifier": "00700",
            "name": "TENCENT",
            "latestPrice": 484.0,
            "multiplier": 1.0
        }"#;
        let position: Position = serde_json::from_str(json).unwrap();
        assert_eq!(position.account, Some("DU123456".to_string()));
        assert_eq!(position.symbol, Some("00700".to_string()));
        assert_eq!(position.position, Some(11600));
        assert_eq!(position.average_cost, Some(631.6747));
        assert_eq!(position.unrealized_pnl_percent, Some(-0.2338));
        assert_eq!(position.contract_id, Some(19083));
        assert_eq!(position.name, Some("TENCENT".to_string()));
        assert_eq!(position.latest_price, Some(484.0));
    }
}
