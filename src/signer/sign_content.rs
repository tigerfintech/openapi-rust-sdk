//! 请求参数排序拼接功能。
//! 按参数名字母序排列所有参数，拼接为 key=value&key=value 格式。

use std::collections::BTreeMap;

/// 按参数名字母序排列所有参数，拼接为 key=value&key=value 格式。
/// BTreeMap 天然保证键的字母序排列。
pub fn get_sign_content(params: &BTreeMap<String, String>) -> String {
    if params.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    for (i, (key, value)) in params.iter().enumerate() {
        if i > 0 {
            result.push('&');
        }
        result.push_str(key);
        result.push('=');
        result.push_str(value);
    }
    result
}

#[path = "sign_content_test.rs"]
#[cfg(test)]
mod sign_content_test;
