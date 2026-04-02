//! 错误码分类测试模块。

use super::*;
use proptest::prelude::*;

// ========== 单元测试 ==========

#[test]
fn test_classify_success() {
    assert_eq!(classify_error_code(0), ErrorCategory::Success);
}

#[test]
fn test_classify_rate_limit() {
    assert_eq!(classify_error_code(5), ErrorCategory::RateLimit);
}

#[test]
fn test_classify_common_param_error() {
    assert_eq!(classify_error_code(1000), ErrorCategory::CommonParamError);
    assert_eq!(classify_error_code(1005), ErrorCategory::CommonParamError);
    assert_eq!(classify_error_code(1009), ErrorCategory::CommonParamError);
}

#[test]
fn test_classify_biz_param_error() {
    assert_eq!(classify_error_code(1010), ErrorCategory::BizParamError);
    assert_eq!(classify_error_code(1050), ErrorCategory::BizParamError);
    assert_eq!(classify_error_code(1099), ErrorCategory::BizParamError);
}

#[test]
fn test_classify_trade_errors() {
    assert_eq!(classify_error_code(1100), ErrorCategory::TradeGlobalError);
    assert_eq!(classify_error_code(1200), ErrorCategory::TradePrimeError);
    assert_eq!(classify_error_code(1300), ErrorCategory::TradeSimulationError);
}

#[test]
fn test_classify_quote_errors() {
    assert_eq!(classify_error_code(2100), ErrorCategory::QuoteStockError);
    assert_eq!(classify_error_code(2200), ErrorCategory::QuoteOptionError);
    assert_eq!(classify_error_code(2300), ErrorCategory::QuoteFutureError);
}

#[test]
fn test_classify_token_error() {
    assert_eq!(classify_error_code(2400), ErrorCategory::TokenError);
}

#[test]
fn test_classify_permission_error() {
    assert_eq!(classify_error_code(4000), ErrorCategory::PermissionError);
}

#[test]
fn test_classify_unknown_error() {
    assert_eq!(classify_error_code(9999), ErrorCategory::UnknownError);
    assert_eq!(classify_error_code(-1), ErrorCategory::UnknownError);
}

// ========== Property 8 属性测试 ==========

/// 已知错误码与预期分类的映射
fn known_error_code_and_category() -> impl Strategy<Value = (i32, ErrorCategory)> {
    prop_oneof![
        Just((0, ErrorCategory::Success)),
        Just((5, ErrorCategory::RateLimit)),
        Just((1000, ErrorCategory::CommonParamError)),
        Just((1005, ErrorCategory::CommonParamError)),
        Just((1009, ErrorCategory::CommonParamError)),
        Just((1010, ErrorCategory::BizParamError)),
        Just((1050, ErrorCategory::BizParamError)),
        Just((1099, ErrorCategory::BizParamError)),
        Just((1100, ErrorCategory::TradeGlobalError)),
        Just((1150, ErrorCategory::TradeGlobalError)),
        Just((1199, ErrorCategory::TradeGlobalError)),
        Just((1200, ErrorCategory::TradePrimeError)),
        Just((1250, ErrorCategory::TradePrimeError)),
        Just((1299, ErrorCategory::TradePrimeError)),
        Just((1300, ErrorCategory::TradeSimulationError)),
        Just((1500, ErrorCategory::TradeSimulationError)),
        Just((2099, ErrorCategory::TradeSimulationError)),
        Just((2100, ErrorCategory::QuoteStockError)),
        Just((2150, ErrorCategory::QuoteStockError)),
        Just((2199, ErrorCategory::QuoteStockError)),
        Just((2200, ErrorCategory::QuoteOptionError)),
        Just((2250, ErrorCategory::QuoteOptionError)),
        Just((2299, ErrorCategory::QuoteOptionError)),
        Just((2300, ErrorCategory::QuoteFutureError)),
        Just((2350, ErrorCategory::QuoteFutureError)),
        Just((2399, ErrorCategory::QuoteFutureError)),
        Just((2400, ErrorCategory::TokenError)),
        Just((3000, ErrorCategory::TokenError)),
        Just((3999, ErrorCategory::TokenError)),
        Just((4000, ErrorCategory::PermissionError)),
        Just((4500, ErrorCategory::PermissionError)),
        Just((4999, ErrorCategory::PermissionError)),
    ]
}

// **Validates: Requirements 8.2, 8.3, 8.4, 8.5**
//
// Feature: multi-language-sdks, Property 8: 错误码分类正确性
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn error_code_classification_correctness(
        (code, expected_category) in known_error_code_and_category()
    ) {
        let actual = classify_error_code(code);
        prop_assert_eq!(actual, expected_category,
            "错误码 {} 的分类应为 {:?}，实际为 {:?}", code, expected_category, actual);
    }
}
