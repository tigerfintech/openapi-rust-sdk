//! 重试策略模块。
//! 实现指数退避重试，交易操作跳过重试。

use std::time::Duration;

/// 交易操作方法名，这些操作不应自动重试
const TRADE_OPERATIONS: &[&str] = &["place_order", "modify_order", "cancel_order"];

/// 重试策略
pub struct RetryPolicy {
    /// 最大重试次数，默认 5
    pub max_retries: u32,
    /// 最大重试总时间，默认 60 秒
    pub max_retry_time: Duration,
    /// 基础退避时间，默认 1 秒
    pub base_delay: Duration,
    /// 最大单次退避时间，默认 16 秒
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 5,
            max_retry_time: Duration::from_secs(60),
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(16),
        }
    }
}

impl RetryPolicy {
    /// 判断指定的 API 方法是否应该重试
    /// 交易操作（place_order、modify_order、cancel_order）跳过重试
    pub fn should_retry(&self, api_method: &str) -> bool {
        !is_trade_operation(api_method)
    }

    /// 计算第 n 次重试的退避等待时间（从 0 开始计数）
    /// 退避公式：min(2^n * base_delay, max_delay)
    pub fn calculate_backoff(&self, retry_count: u32) -> Duration {
        let delay = self.base_delay.mul_f64(2f64.powi(retry_count as i32));
        if delay > self.max_delay {
            self.max_delay
        } else {
            delay
        }
    }
}

/// 判断是否为交易操作
pub fn is_trade_operation(api_method: &str) -> bool {
    TRADE_OPERATIONS.contains(&api_method)
}

#[cfg(test)]
mod tests;
