//! Properties 配置文件解析器。
//!
//! 解析 Java properties 格式的配置文件（`tiger_openapi_config.properties`），支持：
//! - `key=value` 和 `key:value` 键值对
//! - `\` 续行（多行值）
//! - `#` 和 `!` 注释行
//! - 空行忽略

use std::collections::HashMap;
use std::fs;
use crate::error::TigerError;

/// 解析 Java properties 格式的配置文件。
///
/// # 参数
/// - `path`: 配置文件路径
///
/// # 返回
/// 解析后的键值对 HashMap，或配置错误
pub fn parse_properties_file(path: &str) -> Result<HashMap<String, String>, TigerError> {
    let content = fs::read_to_string(path)
        .map_err(|e| TigerError::Config(format!("无法打开配置文件 {}: {}", path, e)))?;
    parse_properties(&content)
}

/// 从字符串内容解析 properties 格式的键值对。
///
/// 支持 `=` 和 `:` 分隔符，`\` 续行，`#` 和 `!` 注释行。
pub fn parse_properties(content: &str) -> Result<HashMap<String, String>, TigerError> {
    let mut props = HashMap::new();
    let mut current_line = String::new();
    let mut continuation = false;

    for line in content.lines() {
        if continuation {
            // 续行：去除前导空格后拼接
            let trimmed = line.trim_start();
            if trimmed.ends_with('\\') {
                current_line.push_str(&trimmed[..trimmed.len() - 1]);
                continue;
            }
            current_line.push_str(trimmed);
            continuation = false;
        } else {
            let trimmed = line.trim();

            // 跳过空行
            if trimmed.is_empty() {
                continue;
            }

            // 跳过注释行
            if trimmed.starts_with('#') || trimmed.starts_with('!') {
                continue;
            }

            // 检查续行
            if trimmed.ends_with('\\') {
                current_line = trimmed[..trimmed.len() - 1].to_string();
                continuation = true;
                continue;
            }

            current_line = trimmed.to_string();
        }

        // 解析键值对
        if let Some((key, value)) = parse_key_value(&current_line) {
            props.insert(key, value);
        }
        current_line.clear();
    }

    // 处理最后一行是续行但文件结束的情况
    if continuation && !current_line.is_empty() {
        if let Some((key, value)) = parse_key_value(&current_line) {
            props.insert(key, value);
        }
    }

    Ok(props)
}

/// 将键值对序列化为 properties 格式字符串。
///
/// 用于属性测试的 round-trip 验证。
pub fn serialize_properties(props: &HashMap<String, String>) -> String {
    let mut lines: Vec<String> = props
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    // 排序以保证确定性输出
    lines.sort();
    lines.join("\n")
}

/// 解析单行键值对，支持 `=` 和 `:` 分隔符。
/// 值中可以包含 `=` 或 `:`，只按第一个分隔符拆分。
fn parse_key_value(line: &str) -> Option<(String, String)> {
    // 找到第一个 = 或 : 的位置
    let eq_idx = line.find('=');
    let colon_idx = line.find(':');

    let sep_idx = match (eq_idx, colon_idx) {
        (Some(e), Some(c)) => Some(e.min(c)),
        (Some(e), None) => Some(e),
        (None, Some(c)) => Some(c),
        (None, None) => None,
    };

    let sep_idx = sep_idx?;

    let key = line[..sep_idx].trim().to_string();
    let value = line[sep_idx + 1..].trim().to_string();

    if key.is_empty() {
        return None;
    }

    Some((key, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::io::Write;

    // ========== 单元测试 ==========

    /// 测试基本键值对解析
    #[test]
    fn test_parse_basic_key_value() {
        let content = "tiger_id=test123\nprivate_key=abc456\n";
        let props = parse_properties(content).unwrap();
        assert_eq!(props.get("tiger_id").unwrap(), "test123");
        assert_eq!(props.get("private_key").unwrap(), "abc456");
    }

    /// 测试冒号分隔符
    #[test]
    fn test_parse_colon_separator() {
        let content = "tiger_id:test123\nprivate_key:abc456\n";
        let props = parse_properties(content).unwrap();
        assert_eq!(props.get("tiger_id").unwrap(), "test123");
        assert_eq!(props.get("private_key").unwrap(), "abc456");
    }

    /// 测试注释行（# 和 !）
    #[test]
    fn test_parse_comments() {
        let content = "# 这是注释\ntiger_id=test123\n! 这也是注释\nprivate_key=abc456\n";
        let props = parse_properties(content).unwrap();
        assert_eq!(props.len(), 2);
        assert_eq!(props.get("tiger_id").unwrap(), "test123");
        assert_eq!(props.get("private_key").unwrap(), "abc456");
    }

    /// 测试空行忽略
    #[test]
    fn test_parse_empty_lines() {
        let content = "\ntiger_id=test123\n\n\nprivate_key=abc456\n\n";
        let props = parse_properties(content).unwrap();
        assert_eq!(props.len(), 2);
    }

    /// 测试续行（反斜杠续行）
    #[test]
    fn test_parse_continuation() {
        let content = "private_key=MIIEvgIBADANBg\\\n  kqhkiG9w0BAQEF\\\n  AASCBKgwggSk\n";
        let props = parse_properties(content).unwrap();
        assert_eq!(
            props.get("private_key").unwrap(),
            "MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSk"
        );
    }

    /// 测试键值对前后空格被去除
    #[test]
    fn test_parse_trim_spaces() {
        let content = "  tiger_id = test123  \n  private_key = abc456  \n";
        let props = parse_properties(content).unwrap();
        assert_eq!(props.get("tiger_id").unwrap(), "test123");
        assert_eq!(props.get("private_key").unwrap(), "abc456");
    }

    /// 测试值中包含等号
    #[test]
    fn test_parse_value_with_equals() {
        let content = "private_key=abc=def=ghi\n";
        let props = parse_properties(content).unwrap();
        assert_eq!(props.get("private_key").unwrap(), "abc=def=ghi");
    }

    /// 测试空内容
    #[test]
    fn test_parse_empty_content() {
        let content = "";
        let props = parse_properties(content).unwrap();
        assert!(props.is_empty());
    }

    /// 测试从文件解析
    #[test]
    fn test_parse_properties_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_rust_config.properties");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "tiger_id=test123").unwrap();
        writeln!(file, "private_key=abc456").unwrap();
        writeln!(file, "account=DU123456").unwrap();
        drop(file);

        let props = super::parse_properties_file(path.to_str().unwrap()).unwrap();
        assert_eq!(props.get("tiger_id").unwrap(), "test123");
        assert_eq!(props.get("private_key").unwrap(), "abc456");
        assert_eq!(props.get("account").unwrap(), "DU123456");

        std::fs::remove_file(&path).ok();
    }

    /// 测试文件不存在时返回错误
    #[test]
    fn test_parse_nonexistent_file() {
        let result = super::parse_properties_file("/nonexistent/path/config.properties");
        assert!(result.is_err());
    }

    // ========== Property 1 属性测试：Properties 配置文件解析 round-trip ==========

    /// 生成有效的 properties 键名（字母数字下划线，非空）
    fn valid_key_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z][a-zA-Z0-9_]{0,19}"
    }

    /// 生成有效的 properties 值（不含特殊字符 #、!、\n、\r、\）
    fn valid_value_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 _./@+\\-]{1,50}"
            .prop_map(|s| s.trim().to_string())
            .prop_filter("值不能为空", |s| !s.is_empty())
    }

    /// **Validates: Requirements 2.8, 10.7**
    ///
    /// Feature: multi-language-sdks, Property 1: Properties 配置文件解析 round-trip
    ///
    /// 对于任意有效的键值对集合，将其序列化为 Java properties 格式后再解析，
    /// 得到的键值对集合应与原始集合等价。
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        #[test]
        fn properties_round_trip(
            pairs in proptest::collection::hash_map(
                valid_key_strategy(),
                valid_value_strategy(),
                1..10
            )
        ) {
            // 序列化为 properties 格式
            let serialized = serialize_properties(&pairs);
            // 解析回来
            let parsed = parse_properties(&serialized).unwrap();
            // 验证等价
            prop_assert_eq!(parsed, pairs);
        }
    }
}
