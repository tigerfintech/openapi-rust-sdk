//! 公共 JSON 解码辅助函数。
//!
//! [`decode_value`] 处理普通值反序列化，以及服务端偶尔返回的「双重编码」JSON 字符串。

use serde_json::Value;

use crate::error::TigerError;

/// 将 [`serde_json::Value`] 反序列化为目标类型 `T`。
///
/// 当普通反序列化失败且值是 JSON 字符串（双重编码）时，先解一层再重试。
pub fn decode_value<T>(v: Value) -> Result<T, TigerError>
where
    T: serde::de::DeserializeOwned,
{
    match serde_json::from_value::<T>(v.clone()) {
        Ok(out) => Ok(out),
        Err(original_err) => {
            if let Value::String(s) = &v {
                return serde_json::from_str::<T>(s).map_err(|e| {
                    TigerError::Parse(format!("decode data (double-encoded) failed: {}", e))
                });
            }
            Err(TigerError::Parse(format!(
                "decode data failed: {}",
                original_err
            )))
        }
    }
}
