//! 枚举类型定义，包含市场、证券类型、货币、订单类型等。

use serde::{Deserialize, Serialize};

/// 市场枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Market {
    #[serde(rename = "ALL")]
    All,
    #[serde(rename = "US")]
    Us,
    #[serde(rename = "HK")]
    Hk,
    #[serde(rename = "CN")]
    Cn,
    #[serde(rename = "SG")]
    Sg,
}

/// 证券类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityType {
    #[serde(rename = "ALL")]
    All,
    #[serde(rename = "STK")]
    Stk,
    #[serde(rename = "OPT")]
    Opt,
    #[serde(rename = "WAR")]
    War,
    #[serde(rename = "IOPT")]
    Iopt,
    #[serde(rename = "FUT")]
    Fut,
    #[serde(rename = "FOP")]
    Fop,
    #[serde(rename = "CASH")]
    Cash,
    #[serde(rename = "MLEG")]
    Mleg,
    #[serde(rename = "FUND")]
    Fund,
}

/// 货币枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Currency {
    #[serde(rename = "ALL")]
    All,
    #[serde(rename = "USD")]
    Usd,
    #[serde(rename = "HKD")]
    Hkd,
    #[serde(rename = "CNH")]
    Cnh,
    #[serde(rename = "SGD")]
    Sgd,
}

/// 订单类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    #[serde(rename = "MKT")]
    Mkt,
    #[serde(rename = "LMT")]
    Lmt,
    #[serde(rename = "STP")]
    Stp,
    #[serde(rename = "STP_LMT")]
    StpLmt,
    #[serde(rename = "TRAIL")]
    Trail,
    #[serde(rename = "AM")]
    Am,
    #[serde(rename = "AL")]
    Al,
    #[serde(rename = "TWAP")]
    Twap,
    #[serde(rename = "VWAP")]
    Vwap,
    #[serde(rename = "OCA")]
    Oca,
}

/// 订单状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    PendingNew,
    Initial,
    Submitted,
    PartiallyFilled,
    Filled,
    Cancelled,
    PendingCancel,
    Inactive,
    Invalid,
}

/// K 线周期枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BarPeriod {
    #[serde(rename = "day")]
    Day,
    #[serde(rename = "week")]
    Week,
    #[serde(rename = "month")]
    Month,
    #[serde(rename = "year")]
    Year,
    #[serde(rename = "1min")]
    Min1,
    #[serde(rename = "5min")]
    Min5,
    #[serde(rename = "15min")]
    Min15,
    #[serde(rename = "30min")]
    Min30,
    #[serde(rename = "60min")]
    Min60,
}

/// 语言枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Language {
    #[serde(rename = "zh_CN")]
    ZhCn,
    #[serde(rename = "zh_TW")]
    ZhTw,
    #[serde(rename = "en_US")]
    EnUs,
}

/// 复权类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuoteRight {
    /// 前复权
    #[serde(rename = "br")]
    Br,
    /// 不复权
    #[serde(rename = "nr")]
    Nr,
}

/// 牌照类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum License {
    #[serde(rename = "TBNZ")]
    Tbnz,
    #[serde(rename = "TBSG")]
    Tbsg,
    #[serde(rename = "TBHK")]
    Tbhk,
    #[serde(rename = "TBAU")]
    Tbau,
    #[serde(rename = "TBUS")]
    Tbus,
}

/// 订单有效期枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeInForce {
    #[serde(rename = "DAY")]
    Day,
    #[serde(rename = "GTC")]
    Gtc,
    #[serde(rename = "OPG")]
    Opg,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Market 枚举测试 ==========

    #[test]
    fn test_market_serialize() {
        assert_eq!(serde_json::to_string(&Market::All).unwrap(), "\"ALL\"");
        assert_eq!(serde_json::to_string(&Market::Us).unwrap(), "\"US\"");
        assert_eq!(serde_json::to_string(&Market::Hk).unwrap(), "\"HK\"");
        assert_eq!(serde_json::to_string(&Market::Cn).unwrap(), "\"CN\"");
        assert_eq!(serde_json::to_string(&Market::Sg).unwrap(), "\"SG\"");
    }

    #[test]
    fn test_market_deserialize() {
        assert_eq!(serde_json::from_str::<Market>("\"ALL\"").unwrap(), Market::All);
        assert_eq!(serde_json::from_str::<Market>("\"US\"").unwrap(), Market::Us);
        assert_eq!(serde_json::from_str::<Market>("\"HK\"").unwrap(), Market::Hk);
        assert_eq!(serde_json::from_str::<Market>("\"CN\"").unwrap(), Market::Cn);
        assert_eq!(serde_json::from_str::<Market>("\"SG\"").unwrap(), Market::Sg);
    }

    // ========== SecurityType 枚举测试 ==========

    #[test]
    fn test_security_type_serialize() {
        assert_eq!(serde_json::to_string(&SecurityType::All).unwrap(), "\"ALL\"");
        assert_eq!(serde_json::to_string(&SecurityType::Stk).unwrap(), "\"STK\"");
        assert_eq!(serde_json::to_string(&SecurityType::Opt).unwrap(), "\"OPT\"");
        assert_eq!(serde_json::to_string(&SecurityType::War).unwrap(), "\"WAR\"");
        assert_eq!(serde_json::to_string(&SecurityType::Iopt).unwrap(), "\"IOPT\"");
        assert_eq!(serde_json::to_string(&SecurityType::Fut).unwrap(), "\"FUT\"");
        assert_eq!(serde_json::to_string(&SecurityType::Fop).unwrap(), "\"FOP\"");
        assert_eq!(serde_json::to_string(&SecurityType::Cash).unwrap(), "\"CASH\"");
        assert_eq!(serde_json::to_string(&SecurityType::Mleg).unwrap(), "\"MLEG\"");
        assert_eq!(serde_json::to_string(&SecurityType::Fund).unwrap(), "\"FUND\"");
    }

    #[test]
    fn test_security_type_deserialize() {
        assert_eq!(serde_json::from_str::<SecurityType>("\"STK\"").unwrap(), SecurityType::Stk);
        assert_eq!(serde_json::from_str::<SecurityType>("\"OPT\"").unwrap(), SecurityType::Opt);
        assert_eq!(serde_json::from_str::<SecurityType>("\"FUT\"").unwrap(), SecurityType::Fut);
        assert_eq!(serde_json::from_str::<SecurityType>("\"FUND\"").unwrap(), SecurityType::Fund);
    }

    // ========== Currency 枚举测试 ==========

    #[test]
    fn test_currency_serialize() {
        assert_eq!(serde_json::to_string(&Currency::All).unwrap(), "\"ALL\"");
        assert_eq!(serde_json::to_string(&Currency::Usd).unwrap(), "\"USD\"");
        assert_eq!(serde_json::to_string(&Currency::Hkd).unwrap(), "\"HKD\"");
        assert_eq!(serde_json::to_string(&Currency::Cnh).unwrap(), "\"CNH\"");
        assert_eq!(serde_json::to_string(&Currency::Sgd).unwrap(), "\"SGD\"");
    }

    #[test]
    fn test_currency_deserialize() {
        assert_eq!(serde_json::from_str::<Currency>("\"USD\"").unwrap(), Currency::Usd);
        assert_eq!(serde_json::from_str::<Currency>("\"HKD\"").unwrap(), Currency::Hkd);
    }

    // ========== OrderType 枚举测试 ==========

    #[test]
    fn test_order_type_serialize() {
        assert_eq!(serde_json::to_string(&OrderType::Mkt).unwrap(), "\"MKT\"");
        assert_eq!(serde_json::to_string(&OrderType::Lmt).unwrap(), "\"LMT\"");
        assert_eq!(serde_json::to_string(&OrderType::Stp).unwrap(), "\"STP\"");
        assert_eq!(serde_json::to_string(&OrderType::StpLmt).unwrap(), "\"STP_LMT\"");
        assert_eq!(serde_json::to_string(&OrderType::Trail).unwrap(), "\"TRAIL\"");
        assert_eq!(serde_json::to_string(&OrderType::Am).unwrap(), "\"AM\"");
        assert_eq!(serde_json::to_string(&OrderType::Al).unwrap(), "\"AL\"");
        assert_eq!(serde_json::to_string(&OrderType::Twap).unwrap(), "\"TWAP\"");
        assert_eq!(serde_json::to_string(&OrderType::Vwap).unwrap(), "\"VWAP\"");
        assert_eq!(serde_json::to_string(&OrderType::Oca).unwrap(), "\"OCA\"");
    }

    #[test]
    fn test_order_type_deserialize() {
        assert_eq!(serde_json::from_str::<OrderType>("\"MKT\"").unwrap(), OrderType::Mkt);
        assert_eq!(serde_json::from_str::<OrderType>("\"STP_LMT\"").unwrap(), OrderType::StpLmt);
        assert_eq!(serde_json::from_str::<OrderType>("\"TWAP\"").unwrap(), OrderType::Twap);
    }

    // ========== OrderStatus 枚举测试 ==========

    #[test]
    fn test_order_status_serialize() {
        assert_eq!(serde_json::to_string(&OrderStatus::PendingNew).unwrap(), "\"PendingNew\"");
        assert_eq!(serde_json::to_string(&OrderStatus::Initial).unwrap(), "\"Initial\"");
        assert_eq!(serde_json::to_string(&OrderStatus::Submitted).unwrap(), "\"Submitted\"");
        assert_eq!(serde_json::to_string(&OrderStatus::PartiallyFilled).unwrap(), "\"PartiallyFilled\"");
        assert_eq!(serde_json::to_string(&OrderStatus::Filled).unwrap(), "\"Filled\"");
        assert_eq!(serde_json::to_string(&OrderStatus::Cancelled).unwrap(), "\"Cancelled\"");
        assert_eq!(serde_json::to_string(&OrderStatus::PendingCancel).unwrap(), "\"PendingCancel\"");
        assert_eq!(serde_json::to_string(&OrderStatus::Inactive).unwrap(), "\"Inactive\"");
        assert_eq!(serde_json::to_string(&OrderStatus::Invalid).unwrap(), "\"Invalid\"");
    }

    #[test]
    fn test_order_status_deserialize() {
        assert_eq!(serde_json::from_str::<OrderStatus>("\"PendingNew\"").unwrap(), OrderStatus::PendingNew);
        assert_eq!(serde_json::from_str::<OrderStatus>("\"Filled\"").unwrap(), OrderStatus::Filled);
        assert_eq!(serde_json::from_str::<OrderStatus>("\"Cancelled\"").unwrap(), OrderStatus::Cancelled);
    }

    // ========== BarPeriod 枚举测试 ==========

    #[test]
    fn test_bar_period_serialize() {
        assert_eq!(serde_json::to_string(&BarPeriod::Day).unwrap(), "\"day\"");
        assert_eq!(serde_json::to_string(&BarPeriod::Week).unwrap(), "\"week\"");
        assert_eq!(serde_json::to_string(&BarPeriod::Month).unwrap(), "\"month\"");
        assert_eq!(serde_json::to_string(&BarPeriod::Year).unwrap(), "\"year\"");
        assert_eq!(serde_json::to_string(&BarPeriod::Min1).unwrap(), "\"1min\"");
        assert_eq!(serde_json::to_string(&BarPeriod::Min5).unwrap(), "\"5min\"");
        assert_eq!(serde_json::to_string(&BarPeriod::Min15).unwrap(), "\"15min\"");
        assert_eq!(serde_json::to_string(&BarPeriod::Min30).unwrap(), "\"30min\"");
        assert_eq!(serde_json::to_string(&BarPeriod::Min60).unwrap(), "\"60min\"");
    }

    #[test]
    fn test_bar_period_deserialize() {
        assert_eq!(serde_json::from_str::<BarPeriod>("\"day\"").unwrap(), BarPeriod::Day);
        assert_eq!(serde_json::from_str::<BarPeriod>("\"1min\"").unwrap(), BarPeriod::Min1);
        assert_eq!(serde_json::from_str::<BarPeriod>("\"60min\"").unwrap(), BarPeriod::Min60);
    }

    // ========== Language 枚举测试 ==========

    #[test]
    fn test_language_serialize() {
        assert_eq!(serde_json::to_string(&Language::ZhCn).unwrap(), "\"zh_CN\"");
        assert_eq!(serde_json::to_string(&Language::ZhTw).unwrap(), "\"zh_TW\"");
        assert_eq!(serde_json::to_string(&Language::EnUs).unwrap(), "\"en_US\"");
    }

    #[test]
    fn test_language_deserialize() {
        assert_eq!(serde_json::from_str::<Language>("\"zh_CN\"").unwrap(), Language::ZhCn);
        assert_eq!(serde_json::from_str::<Language>("\"en_US\"").unwrap(), Language::EnUs);
    }

    // ========== QuoteRight 枚举测试 ==========

    #[test]
    fn test_quote_right_serialize() {
        assert_eq!(serde_json::to_string(&QuoteRight::Br).unwrap(), "\"br\"");
        assert_eq!(serde_json::to_string(&QuoteRight::Nr).unwrap(), "\"nr\"");
    }

    #[test]
    fn test_quote_right_deserialize() {
        assert_eq!(serde_json::from_str::<QuoteRight>("\"br\"").unwrap(), QuoteRight::Br);
        assert_eq!(serde_json::from_str::<QuoteRight>("\"nr\"").unwrap(), QuoteRight::Nr);
    }

    // ========== License 枚举测试 ==========

    #[test]
    fn test_license_serialize() {
        assert_eq!(serde_json::to_string(&License::Tbnz).unwrap(), "\"TBNZ\"");
        assert_eq!(serde_json::to_string(&License::Tbsg).unwrap(), "\"TBSG\"");
        assert_eq!(serde_json::to_string(&License::Tbhk).unwrap(), "\"TBHK\"");
        assert_eq!(serde_json::to_string(&License::Tbau).unwrap(), "\"TBAU\"");
        assert_eq!(serde_json::to_string(&License::Tbus).unwrap(), "\"TBUS\"");
    }

    #[test]
    fn test_license_deserialize() {
        assert_eq!(serde_json::from_str::<License>("\"TBNZ\"").unwrap(), License::Tbnz);
        assert_eq!(serde_json::from_str::<License>("\"TBHK\"").unwrap(), License::Tbhk);
    }

    // ========== TimeInForce 枚举测试 ==========

    #[test]
    fn test_time_in_force_serialize() {
        assert_eq!(serde_json::to_string(&TimeInForce::Day).unwrap(), "\"DAY\"");
        assert_eq!(serde_json::to_string(&TimeInForce::Gtc).unwrap(), "\"GTC\"");
        assert_eq!(serde_json::to_string(&TimeInForce::Opg).unwrap(), "\"OPG\"");
    }

    #[test]
    fn test_time_in_force_deserialize() {
        assert_eq!(serde_json::from_str::<TimeInForce>("\"DAY\"").unwrap(), TimeInForce::Day);
        assert_eq!(serde_json::from_str::<TimeInForce>("\"GTC\"").unwrap(), TimeInForce::Gtc);
        assert_eq!(serde_json::from_str::<TimeInForce>("\"OPG\"").unwrap(), TimeInForce::Opg);
    }
}
