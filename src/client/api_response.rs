//! API 响应解析模块。
//! 解析 API 返回的 JSON 响应，code=0 时返回成功，否则返回错误。

use serde::{Deserialize, Serialize};
use crate::error::TigerError;

/// API 响应结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    /// 状态码（0=成功）
    pub code: i32,
    /// 状态描述
    pub message: String,
    /// 业务数据（原始 JSON）
    pub data: Option<serde_json::Value>,
    /// 服务器时间戳
    pub timestamp: Option<i64>,
}

/// 解析 API 响应 JSON 字符串
/// 当 code 为 0 时返回 ApiResponse；当 code 不为 0 时返回 TigerError::Api
pub fn parse_api_response(body: &[u8]) -> Result<ApiResponse, TigerError> {
    let resp: ApiResponse = serde_json::from_slice(body)
        .map_err(|e| TigerError::Config(format!("解析响应 JSON 失败: {}", e)))?;
    if resp.code != 0 {
        return Err(TigerError::Api {
            code: resp.code,
            message: resp.message,
        });
    }
    Ok(resp)
}

#[cfg(test)]
mod tests;
