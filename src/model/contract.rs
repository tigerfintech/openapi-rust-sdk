//! 合约模型定义和构造工具函数。
//! 字段名词根保持与 API JSON 一致，通过 serde rename 映射到 API JSON 字段名。

use serde::{Deserialize, Serialize};

/// 最小报价单位价格区间
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TickSize {
    /// 起始价格（字符串，如 "0"、"1"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin: Option<String>,
    /// 结束价格（字符串，如 "1"、"Infinity"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    /// 最小报价单位
    #[serde(rename = "tickSize", skip_serializing_if = "Option::is_none")]
    pub tick_size: Option<f64>,
    /// 类型（CLOSED/OPEN）
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

/// 合约模型，字段名词根保持与 API JSON 一致。
/// Rust struct 字段用 snake_case，通过 serde rename 映射到 API JSON 的 camelCase 字段名。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Contract {
    /// 合约 ID
    #[serde(rename = "contractId", skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<i64>,
    /// 标的代码（如 AAPL）
    pub symbol: String,
    /// 证券类型（STK/OPT/FUT/WAR/CASH/FUND 等）
    #[serde(rename = "secType")]
    pub sec_type: String,
    /// 货币
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// 交易所
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange: Option<String>,
    /// 到期日（期权/期货）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    /// 行权价（期权）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike: Option<f64>,
    /// 看涨/看跌（PUT/CALL），保持 API 原始名 right，不改为 put_call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    /// 合约乘数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiplier: Option<f64>,
    /// 合约标识符
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    /// 合约名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 市场
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    /// 是否可交易，保持 API 原始名 tradeable，不改为 trade
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tradeable: Option<bool>,
    /// 合约内部 ID，保持 API 原始名 conid，不改为 contract_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conid: Option<i64>,
    /// 做空保证金比例
    #[serde(rename = "shortMargin", skip_serializing_if = "Option::is_none")]
    pub short_margin: Option<f64>,
    /// 做空初始保证金比例
    #[serde(rename = "shortInitialMargin", skip_serializing_if = "Option::is_none")]
    pub short_initial_margin: Option<f64>,
    /// 做空维持保证金比例
    #[serde(rename = "shortMaintenanceMargin", skip_serializing_if = "Option::is_none")]
    pub short_maintenace_margin: Option<f64>,
    /// 做多初始保证金
    #[serde(rename = "longInitialMargin", skip_serializing_if = "Option::is_none")]
    pub long_initial_margin: Option<f64>,
    /// 做多维持保证金
    #[serde(rename = "longMaintenanceMargin", skip_serializing_if = "Option::is_none")]
    pub long_maintenace_margin: Option<f64>,
    /// 最小报价单位价格区间
    #[serde(rename = "tickSizes", skip_serializing_if = "Option::is_none")]
    pub tick_sizes: Option<Vec<TickSize>>,
    /// 每手数量
    #[serde(rename = "lotSize", skip_serializing_if = "Option::is_none")]
    pub lot_size: Option<f64>,
}

/// 构造股票合约
pub fn stock_contract(symbol: &str, currency: &str) -> Contract {
    Contract {
        contract_id: None,
        symbol: symbol.to_string(),
        sec_type: "STK".to_string(),
        currency: Some(currency.to_string()),
        exchange: None,
        expiry: None,
        strike: None,
        right: None,
        multiplier: None,
        identifier: None,
        name: None,
        market: None,
        tradeable: None,
        conid: None,
        short_margin: None,
        short_initial_margin: None,
        short_maintenace_margin: None,
        long_initial_margin: None,
        long_maintenace_margin: None,
        tick_sizes: None,
        lot_size: None,
    }
}

/// 通过 identifier 构造期权合约
pub fn option_contract(identifier: &str) -> Contract {
    Contract {
        contract_id: None,
        symbol: String::new(),
        sec_type: "OPT".to_string(),
        currency: None,
        exchange: None,
        expiry: None,
        strike: None,
        right: None,
        multiplier: None,
        identifier: Some(identifier.to_string()),
        name: None,
        market: None,
        tradeable: None,
        conid: None,
        short_margin: None,
        short_initial_margin: None,
        short_maintenace_margin: None,
        long_initial_margin: None,
        long_maintenace_margin: None,
        tick_sizes: None,
        lot_size: None,
    }
}

/// 通过各字段构造期权合约
pub fn option_contract_by_symbol(
    symbol: &str,
    expiry: &str,
    strike: f64,
    right: &str,
    currency: &str,
) -> Contract {
    Contract {
        contract_id: None,
        symbol: symbol.to_string(),
        sec_type: "OPT".to_string(),
        currency: Some(currency.to_string()),
        exchange: None,
        expiry: Some(expiry.to_string()),
        strike: Some(strike),
        right: Some(right.to_string()),
        multiplier: None,
        identifier: None,
        name: None,
        market: None,
        tradeable: None,
        conid: None,
        short_margin: None,
        short_initial_margin: None,
        short_maintenace_margin: None,
        long_initial_margin: None,
        long_maintenace_margin: None,
        tick_sizes: None,
        lot_size: None,
    }
}

/// 构造期货合约
pub fn future_contract(symbol: &str, currency: &str, expiry: &str) -> Contract {
    Contract {
        contract_id: None,
        symbol: symbol.to_string(),
        sec_type: "FUT".to_string(),
        currency: Some(currency.to_string()),
        exchange: None,
        expiry: Some(expiry.to_string()),
        strike: None,
        right: None,
        multiplier: None,
        identifier: None,
        name: None,
        market: None,
        tradeable: None,
        conid: None,
        short_margin: None,
        short_initial_margin: None,
        short_maintenace_margin: None,
        long_initial_margin: None,
        long_maintenace_margin: None,
        tick_sizes: None,
        lot_size: None,
    }
}

/// 构造外汇合约
pub fn cash_contract(symbol: &str) -> Contract {
    Contract {
        contract_id: None,
        symbol: symbol.to_string(),
        sec_type: "CASH".to_string(),
        currency: None,
        exchange: None,
        expiry: None,
        strike: None,
        right: None,
        multiplier: None,
        identifier: None,
        name: None,
        market: None,
        tradeable: None,
        conid: None,
        short_margin: None,
        short_initial_margin: None,
        short_maintenace_margin: None,
        long_initial_margin: None,
        long_maintenace_margin: None,
        tick_sizes: None,
        lot_size: None,
    }
}

/// 构造基金合约
pub fn fund_contract(symbol: &str, currency: &str) -> Contract {
    Contract {
        contract_id: None,
        symbol: symbol.to_string(),
        sec_type: "FUND".to_string(),
        currency: Some(currency.to_string()),
        exchange: None,
        expiry: None,
        strike: None,
        right: None,
        multiplier: None,
        identifier: None,
        name: None,
        market: None,
        tradeable: None,
        conid: None,
        short_margin: None,
        short_initial_margin: None,
        short_maintenace_margin: None,
        long_initial_margin: None,
        long_maintenace_margin: None,
        tick_sizes: None,
        lot_size: None,
    }
}

/// 构造窝轮合约
pub fn warrant_contract(
    symbol: &str,
    currency: &str,
    expiry: &str,
    strike: f64,
    right: &str,
) -> Contract {
    Contract {
        contract_id: None,
        symbol: symbol.to_string(),
        sec_type: "WAR".to_string(),
        currency: Some(currency.to_string()),
        exchange: None,
        expiry: Some(expiry.to_string()),
        strike: Some(strike),
        right: Some(right.to_string()),
        multiplier: None,
        identifier: None,
        name: None,
        market: None,
        tradeable: None,
        conid: None,
        short_margin: None,
        short_initial_margin: None,
        short_maintenace_margin: None,
        long_initial_margin: None,
        long_maintenace_margin: None,
        tick_sizes: None,
        lot_size: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ========== Contract JSON 序列化 round-trip 单元测试 ==========

    #[test]
    fn test_contract_json_round_trip_stock() {
        let contract = stock_contract("AAPL", "USD");
        let json = serde_json::to_string(&contract).unwrap();
        let deserialized: Contract = serde_json::from_str(&json).unwrap();
        assert_eq!(contract, deserialized);
    }

    #[test]
    fn test_contract_json_round_trip_option() {
        let contract = option_contract_by_symbol("AAPL", "20251219", 150.0, "CALL", "USD");
        let json = serde_json::to_string(&contract).unwrap();
        let deserialized: Contract = serde_json::from_str(&json).unwrap();
        assert_eq!(contract, deserialized);
    }

    #[test]
    fn test_contract_json_round_trip_future() {
        let contract = future_contract("ES", "USD", "20251219");
        let json = serde_json::to_string(&contract).unwrap();
        let deserialized: Contract = serde_json::from_str(&json).unwrap();
        assert_eq!(contract, deserialized);
    }

    /// 验证 serde rename 映射到 API JSON 字段名
    #[test]
    fn test_contract_serde_rename_field_names() {
        let contract = Contract {
            contract_id: Some(12345),
            symbol: "AAPL".to_string(),
            sec_type: "STK".to_string(),
            currency: Some("USD".to_string()),
            exchange: None,
            expiry: None,
            strike: None,
            right: Some("CALL".to_string()),
            multiplier: None,
            identifier: None,
            name: Some("Apple Inc".to_string()),
            market: Some("US".to_string()),
            tradeable: Some(true),
            conid: Some(99999),
            short_margin: Some(0.25),
            short_initial_margin: None,
            short_maintenace_margin: None,
            long_initial_margin: None,
            long_maintenace_margin: None,
            tick_sizes: None,
            lot_size: Some(1.0),
        };
        let json_value: serde_json::Value = serde_json::to_value(&contract).unwrap();
        let obj = json_value.as_object().unwrap();

        // 验证 serde rename 映射到 API JSON 字段名
        assert!(obj.contains_key("contractId"), "应映射为 contractId");
        assert!(obj.contains_key("secType"), "应映射为 secType");
        assert!(obj.contains_key("lotSize"), "应映射为 lotSize");
        assert!(obj.contains_key("shortMargin"), "应映射为 shortMargin");

        // 验证不改名的字段保持原始名
        assert!(obj.contains_key("right"), "right 字段不应改名");
        assert!(obj.contains_key("tradeable"), "tradeable 字段不应改名");
        assert!(obj.contains_key("conid"), "conid 字段不应改名");
        assert!(obj.contains_key("symbol"), "symbol 字段不应改名");

        // 验证不存在错误的字段名
        assert!(!obj.contains_key("putCall"), "不应出现 putCall");
        assert!(!obj.contains_key("put_call"), "不应出现 put_call");
        assert!(!obj.contains_key("trade"), "不应出现 trade");
        assert!(!obj.contains_key("contract_id"), "不应出现 snake_case 的 contract_id");
        assert!(!obj.contains_key("sec_type"), "不应出现 snake_case 的 sec_type");
    }

    /// 验证从 API JSON 反序列化
    #[test]
    fn test_contract_deserialize_from_api_json() {
        let json = r#"{
            "contractId": 12345,
            "symbol": "AAPL",
            "secType": "STK",
            "currency": "USD",
            "right": "CALL",
            "tradeable": true,
            "conid": 99999,
            "shortMargin": 0.25,
            "lotSize": 1.0,
            "tickSizes": [{"begin": "0", "end": "1", "tickSize": 0.01}]
        }"#;
        let contract: Contract = serde_json::from_str(json).unwrap();
        assert_eq!(contract.contract_id, Some(12345));
        assert_eq!(contract.symbol, "AAPL");
        assert_eq!(contract.sec_type, "STK");
        assert_eq!(contract.right, Some("CALL".to_string()));
        assert_eq!(contract.tradeable, Some(true));
        assert_eq!(contract.conid, Some(99999));
        assert_eq!(contract.short_margin, Some(0.25));
        assert_eq!(contract.lot_size, Some(1.0));
        assert!(contract.tick_sizes.is_some());
        let tick_sizes = contract.tick_sizes.unwrap();
        assert_eq!(tick_sizes.len(), 1);
        assert_eq!(tick_sizes[0].tick_size, Some(0.01));
    }

    /// 验证 skip_serializing_if 跳过 None 字段
    #[test]
    fn test_contract_skip_none_fields() {
        let contract = stock_contract("AAPL", "USD");
        let json_value: serde_json::Value = serde_json::to_value(&contract).unwrap();
        let obj = json_value.as_object().unwrap();

        // 必填字段应存在
        assert!(obj.contains_key("symbol"));
        assert!(obj.contains_key("secType"));

        // None 字段不应出现在 JSON 中
        assert!(!obj.contains_key("contractId"));
        assert!(!obj.contains_key("exchange"));
        assert!(!obj.contains_key("expiry"));
        assert!(!obj.contains_key("strike"));
        assert!(!obj.contains_key("right"));
        assert!(!obj.contains_key("tradeable"));
        assert!(!obj.contains_key("conid"));
    }

    // ========== 合约构造工具函数测试（任务 4.9） ==========

    #[test]
    fn test_stock_contract() {
        let c = stock_contract("AAPL", "USD");
        assert_eq!(c.symbol, "AAPL");
        assert_eq!(c.sec_type, "STK");
        assert_eq!(c.currency, Some("USD".to_string()));
    }

    #[test]
    fn test_option_contract() {
        let c = option_contract("AAPL 20251219 150.0 CALL");
        assert_eq!(c.sec_type, "OPT");
        assert_eq!(c.identifier, Some("AAPL 20251219 150.0 CALL".to_string()));
    }

    #[test]
    fn test_option_contract_by_symbol() {
        let c = option_contract_by_symbol("AAPL", "20251219", 150.0, "CALL", "USD");
        assert_eq!(c.symbol, "AAPL");
        assert_eq!(c.sec_type, "OPT");
        assert_eq!(c.expiry, Some("20251219".to_string()));
        assert_eq!(c.strike, Some(150.0));
        assert_eq!(c.right, Some("CALL".to_string()));
        assert_eq!(c.currency, Some("USD".to_string()));
    }

    #[test]
    fn test_future_contract() {
        let c = future_contract("ES", "USD", "20251219");
        assert_eq!(c.symbol, "ES");
        assert_eq!(c.sec_type, "FUT");
        assert_eq!(c.currency, Some("USD".to_string()));
        assert_eq!(c.expiry, Some("20251219".to_string()));
    }

    #[test]
    fn test_cash_contract() {
        let c = cash_contract("USD.HKD");
        assert_eq!(c.symbol, "USD.HKD");
        assert_eq!(c.sec_type, "CASH");
        assert_eq!(c.currency, None);
    }

    #[test]
    fn test_fund_contract() {
        let c = fund_contract("SPY", "USD");
        assert_eq!(c.symbol, "SPY");
        assert_eq!(c.sec_type, "FUND");
        assert_eq!(c.currency, Some("USD".to_string()));
    }

    #[test]
    fn test_warrant_contract() {
        let c = warrant_contract("00700", "HKD", "20251219", 350.0, "CALL");
        assert_eq!(c.symbol, "00700");
        assert_eq!(c.sec_type, "WAR");
        assert_eq!(c.currency, Some("HKD".to_string()));
        assert_eq!(c.expiry, Some("20251219".to_string()));
        assert_eq!(c.strike, Some(350.0));
        assert_eq!(c.right, Some("CALL".to_string()));
    }

    // ========== Property 7 属性测试：Contract JSON round-trip ==========
    // **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.7**
    // Feature: multi-language-sdks, Property 7: 数据模型 JSON 序列化 round-trip

    /// 生成任意有效的 Contract 对象
    fn arb_contract() -> impl Strategy<Value = Contract> {
        // 分两组生成，避免超过 proptest 元组 12 元素限制
        // 浮点数限制为 2 位小数，避免 JSON round-trip 精度损失
        let group1 = (
            prop::option::of(any::<i64>()),                          // contract_id
            "[A-Z]{1,5}",                                            // symbol
            prop::sample::select(vec!["STK", "OPT", "FUT", "WAR", "CASH", "FUND"]),
            prop::option::of("[A-Z]{3}"),                            // currency
            prop::option::of("[0-9]{8}"),                             // expiry
            prop::option::of((1i64..1000000i64).prop_map(|v| v as f64 / 100.0)), // strike
        );
        let group2 = (
            prop::option::of(prop::sample::select(vec!["PUT", "CALL"])),
            prop::option::of("[A-Za-z ]{1,20}"),                     // name
            prop::option::of(prop::sample::select(vec!["US", "HK", "CN", "SG"])),
            prop::option::of(any::<bool>()),                         // tradeable
            prop::option::of(any::<i64>()),                          // conid
        );
        (group1, group2).prop_map(|((cid, sym, st, cur, exp, strike), (right, name, mkt, trd, conid))| {
            Contract {
                contract_id: cid,
                symbol: sym,
                sec_type: st.to_string(),
                currency: cur,
                exchange: None,
                expiry: exp,
                strike,
                right: right.map(|s| s.to_string()),
                multiplier: None,
                identifier: None,
                name,
                market: mkt.map(|s| s.to_string()),
                tradeable: trd,
                conid,
                short_margin: None,
                short_initial_margin: None,
                short_maintenace_margin: None,
                long_initial_margin: None,
                long_maintenace_margin: None,
                tick_sizes: None,
                lot_size: None,
            }
        })
    }

    proptest! {
        #[test]
        fn prop_contract_json_round_trip(contract in arb_contract()) {
            // 序列化为 JSON
            let json = serde_json::to_string(&contract).unwrap();
            // 反序列化回 Contract
            let deserialized: Contract = serde_json::from_str(&json).unwrap();
            // 验证 round-trip 等价
            prop_assert_eq!(&contract, &deserialized);

            // 验证 JSON 字段名词根与 API JSON 一致
            let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();
            let obj = json_value.as_object().unwrap();

            // 不应出现 snake_case 的 contractId / secType
            prop_assert!(!obj.contains_key("contract_id"), "不应出现 snake_case 的 contract_id");
            prop_assert!(!obj.contains_key("sec_type"), "不应出现 snake_case 的 sec_type");

            // 不应出现改名后的字段
            prop_assert!(!obj.contains_key("putCall"), "不应出现 putCall");
            prop_assert!(!obj.contains_key("put_call"), "不应出现 put_call");
            prop_assert!(!obj.contains_key("trade"), "不应出现 trade");

            // 必填字段应存在
            prop_assert!(obj.contains_key("symbol"));
            prop_assert!(obj.contains_key("secType"));

            // 如果 contract_id 有值，JSON 中应为 contractId
            if contract.contract_id.is_some() {
                prop_assert!(obj.contains_key("contractId"));
            }
        }
    }
}
