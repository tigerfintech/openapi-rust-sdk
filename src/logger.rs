//! 日志模块
//!
//! 使用 tracing crate 提供结构化日志。
//! 通过 subscriber 机制支持自定义日志输出。
//!
//! # 使用示例
//!
//! ```rust
//! use tigeropen::logger;
//!
//! // 初始化默认日志（输出到 stderr）
//! logger::init_default();
//!
//! // 使用 tracing 宏输出日志
//! tracing::info!("SDK 初始化完成");
//! tracing::debug!(tiger_id = "xxx", "发送请求");
//! ```

/// 初始化默认日志 subscriber（输出到 stderr，INFO 级别）
///
/// 使用 tracing-subscriber 的 fmt subscriber。
/// 如果已经设置了全局 subscriber，此函数不会覆盖。
pub fn init_default() {
    let _ = tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .with_env_filter("tigeropen=info")
        .try_init();
}

/// 初始化 DEBUG 级别日志 subscriber
pub fn init_debug() {
    let _ = tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .with_env_filter("tigeropen=debug")
        .try_init();
}

/// 便捷宏：SDK 内部使用的日志宏
///
/// 这些宏直接使用 tracing 的宏，添加 target = "tigeropen"。
#[macro_export]
macro_rules! sdk_debug {
    ($($arg:tt)*) => {
        tracing::debug!(target: "tigeropen", $($arg)*)
    };
}

#[macro_export]
macro_rules! sdk_info {
    ($($arg:tt)*) => {
        tracing::info!(target: "tigeropen", $($arg)*)
    };
}

#[macro_export]
macro_rules! sdk_warn {
    ($($arg:tt)*) => {
        tracing::warn!(target: "tigeropen", $($arg)*)
    };
}

#[macro_export]
macro_rules! sdk_error {
    ($($arg:tt)*) => {
        tracing::error!(target: "tigeropen", $($arg)*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_default_no_panic() {
        // 初始化不应 panic
        init_default();
    }

    #[test]
    fn test_init_debug_no_panic() {
        init_debug();
    }

    #[test]
    fn test_log_macros_no_panic() {
        // 宏调用不应 panic（即使没有 subscriber）
        tracing::debug!(target: "tigeropen", "debug msg");
        tracing::info!(target: "tigeropen", "info msg");
        tracing::warn!(target: "tigeropen", "warn msg");
        tracing::error!(target: "tigeropen", "error msg");
    }
}
