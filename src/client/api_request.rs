//! API 请求构造模块。
//! 构造 API 请求对象，将业务参数序列化为 JSON 字符串作为 biz_content。

use serde::{Deserialize, Serialize};

/// API 请求结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    /// API 方法名（如 "market_state"、"place_order"）
    pub method: String,
    /// 业务参数 JSON 字符串
    pub biz_content: String,
}

impl ApiRequest {
    /// 创建 API 请求
    /// method: API 方法名
    /// biz_content: 业务参数 JSON 字符串
    pub fn new(method: impl Into<String>, biz_content: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            biz_content: biz_content.into(),
        }
    }

    /// 从可序列化的业务参数创建 API 请求
    pub fn from_params<T: Serialize>(method: impl Into<String>, params: &T) -> Result<Self, serde_json::Error> {
        let biz_content = serde_json::to_string(params)?;
        Ok(Self {
            method: method.into(),
            biz_content,
        })
    }
}

#[cfg(test)]
mod tests;
