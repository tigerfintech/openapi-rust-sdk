//! 错误类型定义。

/// SDK 统一错误类型
#[derive(Debug, thiserror::Error)]
pub enum TigerError {
    /// API 业务错误（code != 0 的响应）
    #[error("API error: code={code}, message={message}")]
    Api { code: i32, message: String },

    /// 网络错误
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// 认证错误（私钥格式错误、签名失败）
    #[error("Auth error: {0}")]
    Auth(String),

    /// 配置错误（缺少必填字段、文件不存在、格式错误）
    #[error("Config error: {0}")]
    Config(String),
}
