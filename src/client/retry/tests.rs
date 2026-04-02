//! 重试策略测试模块。

use super::*;
use proptest::prelude::*;

// ========== 单元测试 ==========

#[test]
fn test_default_retry_policy() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.max_retries, 5);
    assert_eq!(policy.max_retry_time, Duration::from_secs(60));
    assert_eq!(policy.base_delay, Duration::from_secs(1));
    assert_eq!(policy.max_delay, Duration::from_secs(16));
}

#[test]
fn test_backoff_sequence() {
    let policy = RetryPolicy::default();
    // 1s → 2s → 4s → 8s → 16s
    assert_eq!(policy.calculate_backoff(0), Duration::from_secs(1));
    assert_eq!(policy.calculate_backoff(1), Duration::from_secs(2));
    assert_eq!(policy.calculate_backoff(2), Duration::from_secs(4));
    assert_eq!(policy.calculate_backoff(3), Duration::from_secs(8));
    assert_eq!(policy.calculate_backoff(4), Duration::from_secs(16));
}

#[test]
fn test_backoff_capped_at_max() {
    let policy = RetryPolicy::default();
    // 超过 max_delay 时应被截断
    assert_eq!(policy.calculate_backoff(5), Duration::from_secs(16));
    assert_eq!(policy.calculate_backoff(10), Duration::from_secs(16));
}

#[test]
fn test_trade_operations_skip_retry() {
    let policy = RetryPolicy::default();
    assert!(!policy.should_retry("place_order"));
    assert!(!policy.should_retry("modify_order"));
    assert!(!policy.should_retry("cancel_order"));
}

#[test]
fn test_non_trade_operations_allow_retry() {
    let policy = RetryPolicy::default();
    assert!(policy.should_retry("market_state"));
    assert!(policy.should_retry("quote_real_time"));
    assert!(policy.should_retry("get_position"));
}

#[test]
fn test_is_trade_operation() {
    assert!(is_trade_operation("place_order"));
    assert!(is_trade_operation("modify_order"));
    assert!(is_trade_operation("cancel_order"));
    assert!(!is_trade_operation("market_state"));
    assert!(!is_trade_operation(""));
}

// ========== Property 9 属性测试 ==========

// **Validates: Requirements 11.3**
//
// Feature: multi-language-sdks, Property 9: 指数退避时间计算
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn exponential_backoff_calculation(
        retry_count in 0u32..20u32,
    ) {
        let policy = RetryPolicy::default();
        let backoff = policy.calculate_backoff(retry_count);
        // 期望值：min(2^n * 1s, 16s)
        let expected_secs = (2u64.pow(retry_count)).min(16);
        let expected = Duration::from_secs(expected_secs);
        prop_assert_eq!(backoff, expected,
            "重试次数 {} 的退避时间应为 {:?}，实际为 {:?}",
            retry_count, expected, backoff);
    }
}

// ========== Property 10 属性测试 ==========

/// 生成 API 方法名（包含交易和非交易操作）
fn api_method_strategy() -> impl Strategy<Value = (String, bool)> {
    prop_oneof![
        // 交易操作 → 应跳过重试
        Just(("place_order".to_string(), true)),
        Just(("modify_order".to_string(), true)),
        Just(("cancel_order".to_string(), true)),
        // 非交易操作 → 应允许重试
        Just(("market_state".to_string(), false)),
        Just(("quote_real_time".to_string(), false)),
        Just(("get_position".to_string(), false)),
        Just(("get_orders".to_string(), false)),
        Just(("get_assets".to_string(), false)),
        Just(("kline".to_string(), false)),
        Just(("option_chain".to_string(), false)),
    ]
}

// **Validates: Requirements 11.4**
//
// Feature: multi-language-sdks, Property 10: 交易操作跳过重试
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn trade_operations_skip_retry(
        (method, is_trade) in api_method_strategy()
    ) {
        let policy = RetryPolicy::default();
        if is_trade {
            // 交易操作应跳过重试
            prop_assert!(!policy.should_retry(&method),
                "交易操作 {} 应跳过重试", method);
            prop_assert!(is_trade_operation(&method),
                "{} 应被识别为交易操作", method);
        } else {
            // 非交易操作应允许重试
            prop_assert!(policy.should_retry(&method),
                "非交易操作 {} 应允许重试", method);
            prop_assert!(!is_trade_operation(&method),
                "{} 不应被识别为交易操作", method);
        }
    }
}
