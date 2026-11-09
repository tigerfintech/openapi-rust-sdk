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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Item {
        name: String,
        value: i32,
    }

    #[test]
    fn test_decode_value_normal_object() {
        let v = serde_json::json!({"name": "test", "value": 42});
        let result: Item = decode_value(v).unwrap();
        assert_eq!(
            result,
            Item {
                name: "test".into(),
                value: 42
            }
        );
    }

    #[test]
    fn test_decode_value_double_encoded_string() {
        // Server occasionally returns data as a JSON string containing the actual JSON object.
        let inner = r#"{"name":"abc","value":7}"#;
        let v = Value::String(inner.to_string());
        let result: Item = decode_value(v).unwrap();
        assert_eq!(
            result,
            Item {
                name: "abc".into(),
                value: 7
            }
        );
    }

    #[test]
    fn test_decode_value_non_string_type_mismatch_returns_parse_error() {
        // A number cannot be deserialized into Item → should return Parse error,
        // and since it's not a Value::String, the double-encode path is skipped.
        let v = serde_json::json!(42);
        let result = decode_value::<Item>(v);
        assert!(result.is_err());
        match result.unwrap_err() {
            TigerError::Parse(msg) => assert!(msg.contains("decode data failed")),
            other => panic!("expected Parse error, got {:?}", other),
        }
    }

    #[test]
    fn test_decode_value_double_encoded_invalid_json_string() {
        // Value is a string but not valid JSON for the target type → Parse error
        // from the double-encoded branch.
        let v = Value::String("not valid json".to_string());
        let result = decode_value::<Item>(v);
        assert!(result.is_err());
        match result.unwrap_err() {
            TigerError::Parse(msg) => assert!(msg.contains("double-encoded")),
            other => panic!("expected Parse (double-encoded) error, got {:?}", other),
        }
    }

    #[test]
    fn test_decode_value_array_to_struct_fails() {
        let v = serde_json::json!([1, 2, 3]);
        let result = decode_value::<Item>(v);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_value_string_to_string_type() {
        // When the target type is String, a Value::String deserializes directly
        // (no double-decode needed).
        let v = Value::String("hello".to_string());
        let result: String = decode_value(v).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_decode_value_integer() {
        let v = serde_json::json!(99);
        let result: i64 = decode_value(v).unwrap();
        assert_eq!(result, 99);
    }
}
