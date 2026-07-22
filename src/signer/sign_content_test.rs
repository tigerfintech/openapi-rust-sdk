#[cfg(test)]
mod tests {
    use crate::signer::get_sign_content;
    use std::collections::BTreeMap;

    #[test]
    fn test_basic_order() {
        let mut params = BTreeMap::new();
        params.insert("tiger_id".to_string(), "test123".to_string());
        params.insert("method".to_string(), "market_state".to_string());
        params.insert("charset".to_string(), "UTF-8".to_string());
        params.insert("sign_type".to_string(), "RSA".to_string());

        let result = get_sign_content(&params);
        assert_eq!(
            result,
            "charset=UTF-8&method=market_state&sign_type=RSA&tiger_id=test123"
        );
    }

    #[test]
    fn test_single_param() {
        let mut params = BTreeMap::new();
        params.insert("key".to_string(), "value".to_string());
        let result = get_sign_content(&params);
        assert_eq!(result, "key=value");
    }

    #[test]
    fn test_empty_map() {
        let params = BTreeMap::new();
        let result = get_sign_content(&params);
        assert_eq!(result, "");
    }

    #[test]
    fn test_alphabetical_order() {
        let mut params = BTreeMap::new();
        params.insert("zebra".to_string(), "z".to_string());
        params.insert("apple".to_string(), "a".to_string());
        params.insert("mango".to_string(), "m".to_string());
        params.insert("banana".to_string(), "b".to_string());

        let result = get_sign_content(&params);
        assert_eq!(result, "apple=a&banana=b&mango=m&zebra=z");
    }

    #[test]
    fn test_full_api_params() {
        let mut params = BTreeMap::new();
        params.insert("tiger_id".to_string(), "20150001".to_string());
        params.insert("method".to_string(), "market_state".to_string());
        params.insert("charset".to_string(), "UTF-8".to_string());
        params.insert("sign_type".to_string(), "RSA".to_string());
        params.insert(
            "timestamp".to_string(),
            "2024-01-01 00:00:00".to_string(),
        );
        params.insert("version".to_string(), "3.0".to_string());
        params.insert(
            "biz_content".to_string(),
            r#"{"market":"US"}"#.to_string(),
        );

        let result = get_sign_content(&params);
        let expected = r#"biz_content={"market":"US"}&charset=UTF-8&method=market_state&sign_type=RSA&tiger_id=20150001&timestamp=2024-01-01 00:00:00&version=3.0"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_special_characters_in_values() {
        let mut params = BTreeMap::new();
        params.insert("key1".to_string(), "value with spaces".to_string());
        params.insert("key2".to_string(), "value=with=equals".to_string());
        params.insert("key3".to_string(), "value&with&ampersand".to_string());

        let result = get_sign_content(&params);
        assert_eq!(
            result,
            "key1=value with spaces&key2=value=with=equals&key3=value&with&ampersand"
        );
    }
}

// ========== Property 5 属性测试：请求参数按字母序排列 ==========
#[cfg(test)]
mod property_tests {
    use crate::signer::get_sign_content;
    use proptest::prelude::*;
    use std::collections::BTreeMap;

    /// 生成有效的参数名（非空，不含 = 和 &）
    fn valid_key() -> impl Strategy<Value = String> {
        "[a-zA-Z][a-zA-Z0-9_]{0,19}".prop_map(|s| s)
    }

    /// 生成有效的参数值（可以包含任意字符，但不为空）
    fn valid_value() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9_ ]{1,50}".prop_map(|s| s)
    }

    // Feature: multi-language-sdks, Property 5: 请求参数按字母序排列
    // **Validates: Requirements 3.3**
    //
    // 对于任意参数名-值的映射（map），get_sign_content 函数输出的字符串中，
    // 参数应严格按参数名的字母序排列，格式为 key1=value1&key2=value2&...
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        #[test]
        fn sign_content_alphabetical_order(
            pairs in prop::collection::vec((valid_key(), valid_value()), 1..10)
        ) {
            let mut params = BTreeMap::new();
            for (k, v) in &pairs {
                params.insert(k.clone(), v.clone());
            }

            let result = get_sign_content(&params);

            // 验证格式：key=value&key=value
            if params.is_empty() {
                prop_assert_eq!(result, "".to_string());
            } else {
                let parts: Vec<&str> = result.split('&').collect();
                prop_assert_eq!(parts.len(), params.len(), "分段数量应等于参数数量");

                // 验证每个分段的格式
                let mut prev_key = String::new();
                for (i, part) in parts.iter().enumerate() {
                    // 只按第一个 = 分割
                    let eq_pos = part.find('=').expect("每个分段应包含 =");
                    let key = &part[..eq_pos];
                    let value = &part[eq_pos + 1..];

                    // 验证键值对与 BTreeMap 中的一致
                    prop_assert_eq!(
                        params.get(key).map(|s| s.as_str()),
                        Some(value),
                        "键 {} 的值应匹配",
                        key
                    );

                    // 验证严格字母序
                    if i > 0 {
                        prop_assert!(
                            key > prev_key.as_str(),
                            "键 {} 应在 {} 之后（字母序）",
                            key,
                            prev_key
                        );
                    }
                    prev_key = key.to_string();
                }
            }
        }
    }
}
