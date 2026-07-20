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

/// 订单状态枚举（对齐 Java SDK OrderStatus.java）
///
/// 服务端 code 映射：Invalid(-2), Initial(-1), PendingCancel(3),
/// Cancelled(4), Submitted(5), Filled(6), Inactive(7), PendingSubmit(8)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    #[serde(rename = "Invalid")]
    Invalid,
    #[serde(rename = "Initial")]
    Initial,
    #[serde(rename = "PendingCancel")]
    PendingCancel,
    #[serde(rename = "Cancelled")]
    Cancelled,
    #[serde(rename = "Submitted")]
    Submitted,
    #[serde(rename = "Filled")]
    Filled,
    #[serde(rename = "Inactive")]
    Inactive,
    #[serde(rename = "PendingSubmit")]
    PendingSubmit,
}

impl OrderStatus {
    /// 返回服务端数字码
    pub fn code(&self) -> i32 {
        match self {
            OrderStatus::Invalid => -2,
            OrderStatus::Initial => -1,
            OrderStatus::PendingCancel => 3,
            OrderStatus::Cancelled => 4,
            OrderStatus::Submitted => 5,
            OrderStatus::Filled => 6,
            OrderStatus::Inactive => 7,
            OrderStatus::PendingSubmit => 8,
        }
    }
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
    #[serde(rename = "TBMS")]
    Tbms,
}

/// 订单排序字段枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderSortBy {
    #[serde(rename = "LATEST_CREATED")]
    LatestCreated,
    #[serde(rename = "LATEST_STATUS_UPDATED")]
    LatestStatusUpdated,
}

/// 账户分部类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SegmentType {
    #[serde(rename = "ALL")]
    All,
    #[serde(rename = "SEC")]
    Sec,
    #[serde(rename = "FUT")]
    Fut,
    #[serde(rename = "FUND")]
    Fund,
}

/// 公司行动类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CorporateActionType {
    #[serde(rename = "split")]
    Split,
    #[serde(rename = "dividend")]
    Dividend,
    #[serde(rename = "earning")]
    Earning,
    #[serde(rename = "symbol_change")]
    SymbolChange,
    #[serde(rename = "delisting")]
    Delisting,
    #[serde(rename = "ipo")]
    Ipo,
}

/// 行业级别枚举（1~4 级）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IndustryLevel {
    #[serde(rename = "GSECTOR")]
    GSector,
    #[serde(rename = "GGROUP")]
    GGroup,
    #[serde(rename = "GIND")]
    GInd,
    #[serde(rename = "GSUBIND")]
    GSubInd,
}

/// 排序方向枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortDirection {
    #[serde(rename = "SortDir_No")]
    No,
    #[serde(rename = "SortDir_Ascend")]
    Ascend,
    #[serde(rename = "SortDir_Descend")]
    Descend,
}

/// 期权分析周期枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptionAnalysisPeriod {
    #[serde(rename = "3year")]
    ThreeYear,
    #[serde(rename = "52week")]
    FiftyTwoWeek,
    #[serde(rename = "26week")]
    TwentySixWeek,
    #[serde(rename = "13week")]
    ThirteenWeek,
}

/// 财报类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FinancialReportPeriod {
    #[serde(rename = "Annual")]
    Annual,
    #[serde(rename = "Quarterly")]
    Quarterly,
    #[serde(rename = "LTM")]
    Ltm,
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
        assert_eq!(serde_json::to_string(&OrderStatus::Invalid).unwrap(), "\"Invalid\"");
        assert_eq!(serde_json::to_string(&OrderStatus::Initial).unwrap(), "\"Initial\"");
        assert_eq!(serde_json::to_string(&OrderStatus::PendingCancel).unwrap(), "\"PendingCancel\"");
        assert_eq!(serde_json::to_string(&OrderStatus::Cancelled).unwrap(), "\"Cancelled\"");
        assert_eq!(serde_json::to_string(&OrderStatus::Submitted).unwrap(), "\"Submitted\"");
        assert_eq!(serde_json::to_string(&OrderStatus::Filled).unwrap(), "\"Filled\"");
        assert_eq!(serde_json::to_string(&OrderStatus::Inactive).unwrap(), "\"Inactive\"");
        assert_eq!(serde_json::to_string(&OrderStatus::PendingSubmit).unwrap(), "\"PendingSubmit\"");
    }

    #[test]
    fn test_order_status_deserialize() {
        assert_eq!(serde_json::from_str::<OrderStatus>("\"Invalid\"").unwrap(), OrderStatus::Invalid);
        assert_eq!(serde_json::from_str::<OrderStatus>("\"Initial\"").unwrap(), OrderStatus::Initial);
        assert_eq!(serde_json::from_str::<OrderStatus>("\"PendingCancel\"").unwrap(), OrderStatus::PendingCancel);
        assert_eq!(serde_json::from_str::<OrderStatus>("\"Cancelled\"").unwrap(), OrderStatus::Cancelled);
        assert_eq!(serde_json::from_str::<OrderStatus>("\"Submitted\"").unwrap(), OrderStatus::Submitted);
        assert_eq!(serde_json::from_str::<OrderStatus>("\"Filled\"").unwrap(), OrderStatus::Filled);
        assert_eq!(serde_json::from_str::<OrderStatus>("\"Inactive\"").unwrap(), OrderStatus::Inactive);
        assert_eq!(serde_json::from_str::<OrderStatus>("\"PendingSubmit\"").unwrap(), OrderStatus::PendingSubmit);
    }

    #[test]
    fn test_order_status_code() {
        assert_eq!(OrderStatus::Invalid.code(), -2);
        assert_eq!(OrderStatus::Initial.code(), -1);
        assert_eq!(OrderStatus::PendingCancel.code(), 3);
        assert_eq!(OrderStatus::Cancelled.code(), 4);
        assert_eq!(OrderStatus::Submitted.code(), 5);
        assert_eq!(OrderStatus::Filled.code(), 6);
        assert_eq!(OrderStatus::Inactive.code(), 7);
        assert_eq!(OrderStatus::PendingSubmit.code(), 8);
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
        assert_eq!(serde_json::to_string(&License::Tbms).unwrap(), "\"TBMS\"");
    }

    #[test]
    fn test_license_deserialize() {
        assert_eq!(serde_json::from_str::<License>("\"TBNZ\"").unwrap(), License::Tbnz);
        assert_eq!(serde_json::from_str::<License>("\"TBHK\"").unwrap(), License::Tbhk);
        assert_eq!(serde_json::from_str::<License>("\"TBMS\"").unwrap(), License::Tbms);
    }

    // ========== 新增枚举测试 ==========

    #[test]
    fn test_new_enums_serialize() {
        assert_eq!(serde_json::to_string(&OrderSortBy::LatestCreated).unwrap(), "\"LATEST_CREATED\"");
        assert_eq!(serde_json::to_string(&OrderSortBy::LatestStatusUpdated).unwrap(), "\"LATEST_STATUS_UPDATED\"");
        assert_eq!(serde_json::to_string(&SegmentType::Sec).unwrap(), "\"SEC\"");
        assert_eq!(serde_json::to_string(&SegmentType::Fund).unwrap(), "\"FUND\"");
        assert_eq!(serde_json::to_string(&CorporateActionType::Split).unwrap(), "\"split\"");
        assert_eq!(serde_json::to_string(&CorporateActionType::Dividend).unwrap(), "\"dividend\"");
        assert_eq!(serde_json::to_string(&IndustryLevel::GSector).unwrap(), "\"GSECTOR\"");
        assert_eq!(serde_json::to_string(&IndustryLevel::GSubInd).unwrap(), "\"GSUBIND\"");
        assert_eq!(serde_json::to_string(&SortDirection::No).unwrap(), "\"SortDir_No\"");
        assert_eq!(serde_json::to_string(&SortDirection::Ascend).unwrap(), "\"SortDir_Ascend\"");
        assert_eq!(serde_json::to_string(&SortDirection::Descend).unwrap(), "\"SortDir_Descend\"");
        assert_eq!(serde_json::to_string(&OptionAnalysisPeriod::ThreeYear).unwrap(), "\"3year\"");
        assert_eq!(serde_json::to_string(&OptionAnalysisPeriod::FiftyTwoWeek).unwrap(), "\"52week\"");
        assert_eq!(serde_json::to_string(&FinancialReportPeriod::Annual).unwrap(), "\"Annual\"");
        assert_eq!(serde_json::to_string(&FinancialReportPeriod::Ltm).unwrap(), "\"LTM\"");
    }

    #[test]
    fn test_new_enums_deserialize() {
        assert_eq!(serde_json::from_str::<OrderSortBy>("\"LATEST_CREATED\"").unwrap(), OrderSortBy::LatestCreated);
        assert_eq!(serde_json::from_str::<SegmentType>("\"SEC\"").unwrap(), SegmentType::Sec);
        assert_eq!(serde_json::from_str::<CorporateActionType>("\"split\"").unwrap(), CorporateActionType::Split);
        assert_eq!(serde_json::from_str::<IndustryLevel>("\"GSECTOR\"").unwrap(), IndustryLevel::GSector);
        assert_eq!(serde_json::from_str::<SortDirection>("\"SortDir_Descend\"").unwrap(), SortDirection::Descend);
        assert_eq!(serde_json::from_str::<OptionAnalysisPeriod>("\"52week\"").unwrap(), OptionAnalysisPeriod::FiftyTwoWeek);
        assert_eq!(serde_json::from_str::<FinancialReportPeriod>("\"LTM\"").unwrap(), FinancialReportPeriod::Ltm);
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
