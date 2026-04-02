//! 错误码分类模块。
//! 根据 API 错误码返回对应的错误分类。

/// 错误分类枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorCategory {
    /// 成功
    Success,
    /// 公共参数错误
    CommonParamError,
    /// 业务参数错误
    BizParamError,
    /// 频率限制
    RateLimit,
    /// 环球账户交易错误
    TradeGlobalError,
    /// 综合账户交易错误
    TradePrimeError,
    /// 模拟账户交易错误
    TradeSimulationError,
    /// 股票行情错误
    QuoteStockError,
    /// 期权行情错误
    QuoteOptionError,
    /// 期货行情错误
    QuoteFutureError,
    /// Token 错误
    TokenError,
    /// 权限错误
    PermissionError,
    /// 服务端错误
    ServerError,
    /// 未知错误
    UnknownError,
}

/// 根据错误码返回对应的错误分类
pub fn classify_error_code(code: i32) -> ErrorCategory {
    match code {
        0 => ErrorCategory::Success,
        5 => ErrorCategory::RateLimit,
        c if (1000..1010).contains(&c) => ErrorCategory::CommonParamError,
        c if (1010..1100).contains(&c) => ErrorCategory::BizParamError,
        c if (1100..1200).contains(&c) => ErrorCategory::TradeGlobalError,
        c if (1200..1300).contains(&c) => ErrorCategory::TradePrimeError,
        c if (1300..2100).contains(&c) => ErrorCategory::TradeSimulationError,
        c if (2100..2200).contains(&c) => ErrorCategory::QuoteStockError,
        c if (2200..2300).contains(&c) => ErrorCategory::QuoteOptionError,
        c if (2300..2400).contains(&c) => ErrorCategory::QuoteFutureError,
        c if (2400..4000).contains(&c) => ErrorCategory::TokenError,
        c if (4000..5000).contains(&c) => ErrorCategory::PermissionError,
        _ => ErrorCategory::UnknownError,
    }
}

#[cfg(test)]
mod tests;
