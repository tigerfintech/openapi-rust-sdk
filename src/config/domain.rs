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

/// 根据动态域名配置和 license 解析服务器地址。
/// 返回 None 表示无法解析（应使用默认地址）。
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

    // 回退到 COMMON
    if let Some(url) = domain_conf.get(DOMAIN_KEY_COMMON).and_then(|v| v.as_str()) {
        return Some(format!("{}{}", url, GATEWAY_SUFFIX));
    }

    None
}
