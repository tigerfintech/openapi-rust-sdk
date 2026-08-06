//! 动态域名获取模块
//!
//! 从域名花园获取动态域名配置，用于 SDK 初始化时自动选择最优服务器地址。
//! 使用 ureq（纯同步 HTTP 库），避免与 tokio async runtime 冲突。

use std::collections::HashMap;

/// 域名花园地址
const DOMAIN_GARDEN_ADDRESS: &str = "https://cg.play-analytics.com";
/// 动态域名查询超时（秒）
const DOMAIN_QUERY_TIMEOUT_SECS: u64 = 1;
/// TBUS 牌照标识
const LICENSE_TBUS: &str = "TBUS";
/// COMMON 域名 key
const DOMAIN_KEY_COMMON: &str = "COMMON";
/// gateway 后缀
const GATEWAY_SUFFIX: &str = "/gateway";

/// 从域名花园获取动态域名配置。
/// 失败时返回空 HashMap（静默回退）。
///
/// 使用 ureq 同步 HTTP 客户端，可安全在 tokio runtime 内调用。
pub fn query_domains(license: Option<&str>) -> HashMap<String, serde_json::Value> {
    do_query_domains(license).unwrap_or_default()
}

fn do_query_domains(license: Option<&str>) -> Option<HashMap<String, serde_json::Value>> {
    let mut url = DOMAIN_GARDEN_ADDRESS.to_string();
    if license == Some(LICENSE_TBUS) {
        url.push_str("?appName=tradeup");
    }

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(DOMAIN_QUERY_TIMEOUT_SECS))
        .timeout_read(std::time::Duration::from_secs(DOMAIN_QUERY_TIMEOUT_SECS))
        .build();

    let resp: serde_json::Value = agent.get(&url).call().ok()?.into_json().ok()?;

    let items = resp.get("items")?.as_array()?;
    let first = items.first()?;
    let openapi = first.get("openapi")?;
    let conf: HashMap<String, serde_json::Value> = serde_json::from_value(openapi.clone()).ok()?;

    Some(conf)
}

/// Resolve quote server URL from dynamic domain config.
/// Uses `{LICENSE}-QUOTE` key, falls back to COMMON, returns None if unavailable.
pub fn resolve_dynamic_quote_server_url(
    domain_conf: &HashMap<String, serde_json::Value>,
    license: Option<&str>,
) -> Option<String> {
    if domain_conf.is_empty() {
        return None;
    }

    // Try {LICENSE}-QUOTE key first
    if let Some(lic) = license {
        let quote_key = format!("{}-QUOTE", lic);
        if let Some(url) = domain_conf.get(&quote_key).and_then(|v| v.as_str()) {
            return Some(format!("{}{}", url, GATEWAY_SUFFIX));
        }
    }

    // Fall back to COMMON
    if let Some(url) = domain_conf.get(DOMAIN_KEY_COMMON).and_then(|v| v.as_str()) {
        return Some(format!("{}{}", url, GATEWAY_SUFFIX));
    }

    None
}

/// Resolve trade server URL from dynamic domain config and license.
/// Returns None when resolution fails (caller should use default URL).
pub fn resolve_dynamic_server_url(
    domain_conf: &HashMap<String, serde_json::Value>,
    license: Option<&str>,
) -> Option<String> {
    if domain_conf.is_empty() {
        return None;
    }

    let key = license.unwrap_or(DOMAIN_KEY_COMMON);

    if let Some(url) = domain_conf.get(key).and_then(|v| v.as_str()) {
        return Some(format!("{}{}", url, GATEWAY_SUFFIX));
    }

    // Fall back to COMMON
    if let Some(url) = domain_conf.get(DOMAIN_KEY_COMMON).and_then(|v| v.as_str()) {
        return Some(format!("{}{}", url, GATEWAY_SUFFIX));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_conf(pairs: &[(&str, &str)]) -> HashMap<String, serde_json::Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.to_string())))
            .collect()
    }

    // ── resolve_dynamic_server_url ──

    #[test]
    fn test_resolve_server_url_empty_map_returns_none() {
        let conf = HashMap::new();
        assert_eq!(resolve_dynamic_server_url(&conf, None), None);
    }

    #[test]
    fn test_resolve_server_url_license_key() {
        let conf = make_conf(&[("TBNZ", "https://tbnz.example.com")]);
        let url = resolve_dynamic_server_url(&conf, Some("TBNZ")).unwrap();
        assert_eq!(url, "https://tbnz.example.com/gateway");
    }

    #[test]
    fn test_resolve_server_url_falls_back_to_common() {
        let conf = make_conf(&[("COMMON", "https://common.example.com")]);
        // license not present in map → fall back to COMMON
        let url = resolve_dynamic_server_url(&conf, Some("TBNZ")).unwrap();
        assert_eq!(url, "https://common.example.com/gateway");
    }

    #[test]
    fn test_resolve_server_url_no_license_uses_common_key() {
        let conf = make_conf(&[("COMMON", "https://common.example.com")]);
        let url = resolve_dynamic_server_url(&conf, None).unwrap();
        assert_eq!(url, "https://common.example.com/gateway");
    }

    #[test]
    fn test_resolve_server_url_license_present_takes_priority_over_common() {
        let conf = make_conf(&[
            ("TBNZ", "https://tbnz.example.com"),
            ("COMMON", "https://common.example.com"),
        ]);
        let url = resolve_dynamic_server_url(&conf, Some("TBNZ")).unwrap();
        assert_eq!(url, "https://tbnz.example.com/gateway");
    }

    #[test]
    fn test_resolve_server_url_non_string_value_ignored() {
        let mut conf = HashMap::new();
        conf.insert("TBNZ".to_string(), serde_json::json!(42));
        // license key present but not a string → fall back to COMMON (also missing) → None
        assert_eq!(resolve_dynamic_server_url(&conf, Some("TBNZ")), None);
    }

    #[test]
    fn test_resolve_server_url_missing_key_and_missing_common_returns_none() {
        let conf = make_conf(&[("OTHER", "https://other.example.com")]);
        assert_eq!(resolve_dynamic_server_url(&conf, Some("TBNZ")), None);
    }

    // ── resolve_dynamic_quote_server_url ──

    #[test]
    fn test_resolve_quote_url_empty_map_returns_none() {
        let conf = HashMap::new();
        assert_eq!(resolve_dynamic_quote_server_url(&conf, None), None);
    }

    #[test]
    fn test_resolve_quote_url_license_quote_key() {
        let conf = make_conf(&[("TBNZ-QUOTE", "https://quote.tbnz.example.com")]);
        let url = resolve_dynamic_quote_server_url(&conf, Some("TBNZ")).unwrap();
        assert_eq!(url, "https://quote.tbnz.example.com/gateway");
    }

    #[test]
    fn test_resolve_quote_url_falls_back_to_common() {
        let conf = make_conf(&[("COMMON", "https://common.example.com")]);
        let url = resolve_dynamic_quote_server_url(&conf, Some("TBNZ")).unwrap();
        assert_eq!(url, "https://common.example.com/gateway");
    }

    #[test]
    fn test_resolve_quote_url_no_license_falls_back_to_common() {
        let conf = make_conf(&[("COMMON", "https://common.example.com")]);
        let url = resolve_dynamic_quote_server_url(&conf, None).unwrap();
        assert_eq!(url, "https://common.example.com/gateway");
    }

    #[test]
    fn test_resolve_quote_url_license_quote_takes_priority_over_common() {
        let conf = make_conf(&[
            ("TBNZ-QUOTE", "https://quote.tbnz.example.com"),
            ("COMMON", "https://common.example.com"),
        ]);
        let url = resolve_dynamic_quote_server_url(&conf, Some("TBNZ")).unwrap();
        assert_eq!(url, "https://quote.tbnz.example.com/gateway");
    }

    #[test]
    fn test_resolve_quote_url_missing_all_returns_none() {
        let conf = make_conf(&[("OTHER", "https://other.example.com")]);
        assert_eq!(resolve_dynamic_quote_server_url(&conf, Some("TBNZ")), None);
    }

    #[test]
    fn test_resolve_quote_url_non_string_value_ignored() {
        let mut conf = HashMap::new();
        conf.insert("TBNZ-QUOTE".to_string(), serde_json::json!(null));
        conf.insert("COMMON".to_string(), serde_json::Value::String("https://c.example.com".into()));
        let url = resolve_dynamic_quote_server_url(&conf, Some("TBNZ")).unwrap();
        assert_eq!(url, "https://c.example.com/gateway");
    }
}
