//! API 请求构造模块。
//! 构造 API 请求对象，将业务参数序列化为 JSON 字符串作为 biz_content。

use serde::{Deserialize, Serialize};

/// API request struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    /// API method name (e.g. "market_state", "place_order")
    pub method: String,
    /// Business parameters JSON string
    pub biz_content: String,
    /// Optional per-request API version override (e.g. "3.0")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl ApiRequest {
    /// Create an API request
    /// method: API method name
    /// biz_content: business parameters JSON string
    pub fn new(method: impl Into<String>, biz_content: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            biz_content: biz_content.into(),
            version: None,
        }
    }

    /// Create an API request with a specific API version
    pub fn with_version(method: impl Into<String>, biz_content: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            biz_content: biz_content.into(),
            version: Some(version.into()),
        }
    }

    /// Create an API request from serializable business parameters
    pub fn from_params<T: Serialize>(method: impl Into<String>, params: &T) -> Result<Self, serde_json::Error> {
        let biz_content = serde_json::to_string(params)?;
        Ok(Self {
            method: method.into(),
            biz_content,
            version: None,
        })
    }
}

#[cfg(test)]
mod tests;
