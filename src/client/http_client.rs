//! HttpClient wraps HTTP requests, signing, retry, and timeout.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::config::client_config::ClientConfig;
use crate::config::token_manager::TokenManager;
use crate::error::TigerError;
use crate::signer::{get_sign_content, sign_with_rsa, verify_with_rsa};
use super::api_request::ApiRequest;
use super::api_response::{ApiResponse, parse_api_response};
use super::retry::RetryPolicy;

/// User-Agent 前缀
const USER_AGENT_PREFIX: &str = "openapi-rust-sdk-";
/// 默认字符集
const DEFAULT_CHARSET: &str = "UTF-8";
/// 默认签名类型
const DEFAULT_SIGN_TYPE: &str = "RSA";
/// 默认 API 版本
const DEFAULT_VERSION: &str = "2.0";

/// Tiger OpenAPI token 刷新接口方法名
const METHOD_TOKEN_REFRESH: &str = "user_token_refresh";

/// HttpClient wraps HTTP requests, signing, retry, and timeout.
///
/// The client posts to either `config.server_url` (default, for trade endpoints)
/// or `config.quote_server_url` (when constructed via [`HttpClient::with_quote_server`]).
///
/// When `config.token_refresh_duration > Duration::ZERO`, a background tokio task is
/// started automatically and can be stopped with [`HttpClient::close`].
pub struct HttpClient {
    config: Arc<std::sync::RwLock<ClientConfig>>,
    client: reqwest::Client,
    retry_policy: RetryPolicy,
    /// Override POST target. When None, use `config.server_url`.
    url_override: Option<String>,
    /// Internal TokenManager created when auto-refresh is active (started by new()).
    token_manager: Option<Arc<TokenManager>>,
}

impl HttpClient {
    /// Create HttpClient instance (posts to `config.server_url`).
    ///
    /// If `config.token_refresh_duration > 0`, background token auto-refresh is
    /// started automatically. Call [`close`] when the client is no longer needed.
    pub fn new(config: ClientConfig) -> Self {
        let url_override = None;
        Self::build(config, url_override)
    }

    /// Create HttpClient wired to the quote server URL.
    ///
    /// Use this variant when constructing a [`crate::quote::QuoteClient`] so
    /// quote requests go to `config.quote_server_url` instead of the default
    /// `config.server_url` (which is used for trade endpoints).
    ///
    /// Note: auto-refresh is NOT started for quote clients to avoid duplicate
    /// background tasks. The primary HttpClient owns the refresh lifecycle.
    pub fn with_quote_server(config: ClientConfig) -> Self {
        let override_url = config.quote_server_url.clone();
        // Suppress auto-start; primary HttpClient already owns the refresh goroutine.
        let mut cfg = config;
        cfg.token_refresh_duration = None;
        Self::build(cfg, Some(override_url))
    }

    /// Create HttpClient with a custom reqwest::Client (for testing)
    pub fn with_client(config: ClientConfig, client: reqwest::Client) -> Self {
        let config = Arc::new(std::sync::RwLock::new(config));
        Self {
            config,
            client,
            retry_policy: RetryPolicy::default(),
            url_override: None,
            token_manager: None,
        }
    }

    fn build(config: ClientConfig, url_override: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap_or_default();

        let refresh_duration = config.token_refresh_duration;
        let check_interval = config.token_check_interval;
        let token_writer = config.token_writer.clone();
        let token_loader = config.token_loader.clone();

        let config = Arc::new(std::sync::RwLock::new(config));

        let mut hc = Self {
            config: Arc::clone(&config),
            client,
            retry_policy: RetryPolicy::default(),
            url_override,
            token_manager: None,
        };

        // Auto-start token refresh when token_refresh_duration is configured
        if let Some(dur) = refresh_duration {
            if !dur.is_zero() {
                let interval_secs = check_interval
                    .map(|d| d.as_secs())
                    .unwrap_or(300);
                let interval_secs = if interval_secs == 0 { 300 } else { interval_secs };
                let refresh_secs = dur.as_secs() as i64;

                let mut tm = TokenManager::with_refresh_duration(None, refresh_secs);
                tm.set_check_interval(interval_secs);

                if let Some(writer) = token_writer {
                    tm.set_token_writer(move |t| writer(t));
                }
                if let Some(loader) = token_loader {
                    tm.set_token_loader(move || loader());
                }

                // Sync current token into the manager so ShouldTokenRefresh works immediately
                let current_token = hc.config.read().unwrap().token.clone().unwrap_or_default();
                if !current_token.is_empty() {
                    let _ = tm.set_token(&current_token);
                }

                let config_ref = Arc::clone(&hc.config);
                let tm = Arc::new(tm);
                let tm_weak = Arc::downgrade(&tm);
                let tm_clone = Arc::clone(&tm);

                tm.start_auto_refresh(move || {
                    let config_inner = Arc::clone(&config_ref);
                    let _tm_weak = tm_weak.clone();
                    async move {
                        // We need an HttpClient-like helper here; we'll call the API directly
                        // by cloning the config snapshot.
                        let cfg = config_inner.read().unwrap().clone();
                        match query_token_from_config(&cfg).await {
                            Ok(new_token) => {
                                config_inner.write().unwrap().token = Some(new_token.clone());
                                Ok(new_token)
                            }
                            Err(e) => Err(e),
                        }
                    }
                });

                hc.token_manager = Some(tm_clone);
            }
        }

        hc
    }

    /// Stop the background token auto-refresh task (if running).
    /// Call this when the HttpClient is no longer needed.
    pub fn close(&self) {
        if let Some(ref tm) = self.token_manager {
            tm.stop_auto_refresh();
        }
    }

    /// Get User-Agent string
    pub fn user_agent() -> String {
        format!("{}{}", USER_AGENT_PREFIX, crate::VERSION)
    }

    /// Build common request parameters.
    /// `version` allows per-request API version override; defaults to DEFAULT_VERSION.
    fn build_common_params(&self, api_method: &str, biz_content: &str, version: Option<&str>) -> BTreeMap<String, String> {
        let config = self.config.read().unwrap();
        let mut params = BTreeMap::new();
        params.insert("tiger_id".to_string(), config.tiger_id.clone());
        params.insert("method".to_string(), api_method.to_string());
        params.insert("charset".to_string(), DEFAULT_CHARSET.to_string());
        params.insert("sign_type".to_string(), DEFAULT_SIGN_TYPE.to_string());
        params.insert("timestamp".to_string(), chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        params.insert("version".to_string(), version.unwrap_or(DEFAULT_VERSION).to_string());
        params.insert("biz_content".to_string(), biz_content.to_string());
        if !config.device_id.is_empty() {
            params.insert("device_id".to_string(), config.device_id.clone());
        }
        params
    }

    /// Sign the parameters
    fn sign_params(&self, params: &BTreeMap<String, String>) -> Result<String, TigerError> {
        let content = get_sign_content(params);
        let private_key = self.config.read().unwrap().private_key.clone();
        sign_with_rsa(&private_key, &content)
    }

    /// Execute a structured API request, returning a parsed ApiResponse
    pub async fn execute_request(&self, request: &ApiRequest) -> Result<ApiResponse, TigerError> {
        let mut params = self.build_common_params(&request.method, &request.biz_content, request.version.as_deref());
        let sign = self.sign_params(&params)?;
        params.insert("sign".to_string(), sign);

        let timestamp = params.get("timestamp").cloned().unwrap_or_default();

        let max_attempts = if self.retry_policy.should_retry(&request.method) {
            self.retry_policy.max_retries + 1
        } else {
            1
        };

        let mut last_err = None;
        let deadline = std::time::Instant::now() + self.retry_policy.max_retry_time;
        for attempt in 0..max_attempts {
            if attempt > 0 {
                let backoff = self.retry_policy.calculate_backoff(attempt - 1);
                if std::time::Instant::now() + backoff > deadline {
                    break;
                }
                tokio::time::sleep(backoff).await;
            }

            match self.do_http_post(&params).await {
                Ok(body) => {
                    self.verify_response_sign(&body, &timestamp)?;
                    return parse_api_response(&body);
                }
                Err(e) => {
                    last_err = Some(e);
                    if !self.retry_policy.should_retry(&request.method) {
                        return Err(last_err.unwrap());
                    }
                }
            }
        }

        Err(last_err.unwrap())
    }

    /// Generic API call method
    /// api_method: API method name (e.g. "market_state", "place_order")
    /// request_json: raw biz_content JSON string
    /// Returns raw response JSON string without any parsing
    pub async fn execute(&self, api_method: &str, request_json: &str) -> Result<String, TigerError> {
        // Parameter validation
        if api_method.is_empty() {
            return Err(TigerError::Config("api_method 不能为空".to_string()));
        }
        // Validate request_json is valid JSON
        if serde_json::from_str::<serde_json::Value>(request_json).is_err() {
            return Err(TigerError::Config("request_json 不是有效的 JSON".to_string()));
        }

        let mut params = self.build_common_params(api_method, request_json, None);
        let sign = self.sign_params(&params)?;
        params.insert("sign".to_string(), sign);

        let timestamp = params.get("timestamp").cloned().unwrap_or_default();

        let max_attempts = if self.retry_policy.should_retry(api_method) {
            self.retry_policy.max_retries + 1
        } else {
            1
        };

        let mut last_err = None;
        let deadline = std::time::Instant::now() + self.retry_policy.max_retry_time;
        for attempt in 0..max_attempts {
            if attempt > 0 {
                let backoff = self.retry_policy.calculate_backoff(attempt - 1);
                if std::time::Instant::now() + backoff > deadline {
                    break;
                }
                tokio::time::sleep(backoff).await;
            }

            match self.do_http_post(&params).await {
                Ok(body) => {
                    self.verify_response_sign(&body, &timestamp)?;
                    return String::from_utf8(body)
                        .map_err(|e| TigerError::Config(format!("响应体非 UTF-8: {}", e)));
                }
                Err(e) => {
                    last_err = Some(e);
                    if !self.retry_policy.should_retry(api_method) {
                        return Err(last_err.unwrap());
                    }
                }
            }
        }

        Err(last_err.unwrap())
    }

    /// Verify the response signature using the tiger public key.
    /// Extracts the `sign` field from the response JSON, then verifies it
    /// against the request timestamp using SHA1WithRSA.
    fn verify_response_sign(&self, body: &[u8], timestamp: &str) -> Result<(), TigerError> {
        let json: serde_json::Value = serde_json::from_slice(body)
            .map_err(|e| TigerError::Config(format!("failed to parse response JSON for sign verification: {}", e)))?;

        if let Some(sign) = json.get("sign").and_then(|s| s.as_str()) {
            if !sign.is_empty() {
                let tiger_public_key = self.config.read().unwrap().tiger_public_key.clone();
                verify_with_rsa(&tiger_public_key, timestamp, sign)?;
            }
        }
        Ok(())
    }

    /// Send HTTP POST request
    async fn do_http_post(&self, params: &BTreeMap<String, String>) -> Result<Vec<u8>, TigerError> {
        let (url, token) = {
            let config = self.config.read().unwrap();
            let url = self.url_override.as_deref().unwrap_or(&config.server_url).to_string();
            let token = config.token.clone();
            (url, token)
        };

        let mut request = self.client
            .post(&url)
            .header("Content-Type", "application/json;charset=UTF-8")
            .header("User-Agent", Self::user_agent());

        if let Some(ref t) = token {
            if !t.is_empty() {
                request = request.header("Authorization", t.as_str());
            }
        }

        let resp = request
            .json(params)
            .send()
            .await?;

        let body = resp.bytes().await?;
        Ok(body.to_vec())
    }

    // ──────────────────────────────── token management ────────────────────────────────

    /// Call the `user_token_refresh` API and return the new token string.
    /// This method is read-only: it does NOT modify `config.token`.
    pub async fn query_token(&self) -> Result<String, TigerError> {
        let biz_content = serde_json::json!({}).to_string();
        let mut params = self.build_common_params(METHOD_TOKEN_REFRESH, &biz_content, None);
        let sign = self.sign_params(&params)?;
        params.insert("sign".to_string(), sign);

        let body = self.do_http_post(&params).await?;
        let json: serde_json::Value = serde_json::from_slice(&body)
            .map_err(|e| TigerError::Config(format!("解析 token 响应失败: {}", e)))?;

        let code = json.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
        if code != 0 {
            let msg = json.get("message").and_then(|v| v.as_str()).unwrap_or("unknown");
            return Err(TigerError::Config(format!("token 刷新接口返回错误: code={}, message={}", code, msg)));
        }

        let token = json
            .get("data")
            .and_then(|d| d.get("token"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        if token.is_empty() {
            return Err(TigerError::Config("服务端返回空 token".to_string()));
        }

        Ok(token.to_string())
    }

    /// Refresh the token: call the API, update `config.token`, optionally persist
    /// via `token_manager`.
    ///
    /// - If an internal TokenManager was created by `new()` (auto-start), its in-memory
    ///   token is kept in sync via `sync_token` so `should_token_refresh` stays accurate.
    /// - If `token_manager` is `Some`, the new token is written there (file + callback).
    pub async fn refresh_token(&self, token_manager: Option<&TokenManager>) -> Result<(), TigerError> {
        let new_token = self.query_token().await?;
        tracing::info!("[token] refreshed (new len={})", new_token.len());

        self.config.write().unwrap().token = Some(new_token.clone());

        // Keep internal manager in sync (not the same as the caller-supplied one)
        if let Some(ref internal_tm) = self.token_manager {
            let is_same = token_manager
                .map(|tm| std::ptr::eq(tm as *const _, internal_tm.as_ref() as *const _))
                .unwrap_or(false);
            if !is_same {
                internal_tm.sync_token(&new_token);
            }
        }

        if let Some(tm) = token_manager {
            tm.set_token(&new_token)?;
        }
        Ok(())
    }

    /// Manually start (or restart) background token auto-refresh.
    ///
    /// `token_manager` can be `None`: an in-memory-only `TokenManager` is created
    /// internally, suitable when the token is set directly in config rather than loaded
    /// from a file. Pass `opts` to configure the internal manager (e.g. refresh duration,
    /// check interval, writer callback).
    ///
    /// When `token_manager` is `Some`, the supplied manager is used directly and `opts`
    /// are ignored.
    ///
    /// Returns `Arc<TokenManager>` so the caller can later call `stop_auto_refresh`.
    pub fn start_token_auto_refresh(
        &self,
        token_manager: Option<Arc<TokenManager>>,
        refresh_duration_secs: i64,
        check_interval_secs: u64,
        token_writer: Option<Box<dyn Fn(String) + Send + Sync + 'static>>,
    ) -> Arc<TokenManager> {
        let tm = if let Some(existing) = token_manager {
            existing
        } else {
            let mut new_tm = TokenManager::with_refresh_duration(None, refresh_duration_secs);
            new_tm.set_check_interval(check_interval_secs);
            if let Some(writer) = token_writer {
                new_tm.set_token_writer(writer);
            }
            // Sync current config token into manager
            let current_token = self.config.read().unwrap().token.clone().unwrap_or_default();
            if !current_token.is_empty() {
                let _ = new_tm.set_token(&current_token);
            }
            Arc::new(new_tm)
        };

        let config_ref = Arc::clone(&self.config);
        tm.start_auto_refresh(move || {
            let config_inner = Arc::clone(&config_ref);
            async move {
                let cfg = config_inner.read().unwrap().clone();
                match query_token_from_config(&cfg).await {
                    Ok(new_token) => {
                        config_inner.write().unwrap().token = Some(new_token.clone());
                        Ok(new_token)
                    }
                    Err(e) => Err(e),
                }
            }
        });

        tm
    }
}

impl Drop for HttpClient {
    fn drop(&mut self) {
        self.close();
    }
}

// ──────────────────────────────── free helpers ────────────────────────────────

/// Standalone helper: call user_token_refresh using a config snapshot.
/// Used inside the spawned tokio task (avoids borrowing self).
async fn query_token_from_config(config: &ClientConfig) -> Result<String, TigerError> {
    let client = reqwest::Client::builder()
        .timeout(config.timeout)
        .build()
        .map_err(|e| TigerError::Config(format!("创建 HTTP client 失败: {}", e)))?;

    let biz_content = serde_json::json!({}).to_string();
    let mut params: BTreeMap<String, String> = BTreeMap::new();
    params.insert("tiger_id".to_string(), config.tiger_id.clone());
    params.insert("method".to_string(), METHOD_TOKEN_REFRESH.to_string());
    params.insert("charset".to_string(), DEFAULT_CHARSET.to_string());
    params.insert("sign_type".to_string(), DEFAULT_SIGN_TYPE.to_string());
    params.insert("timestamp".to_string(), chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    params.insert("version".to_string(), DEFAULT_VERSION.to_string());
    params.insert("biz_content".to_string(), biz_content);
    if !config.device_id.is_empty() {
        params.insert("device_id".to_string(), config.device_id.clone());
    }

    let content = get_sign_content(&params);
    let sign = sign_with_rsa(&config.private_key, &content)?;
    params.insert("sign".to_string(), sign);

    let mut request = client
        .post(&config.server_url)
        .header("Content-Type", "application/json;charset=UTF-8")
        .header("User-Agent", format!("{}{}", USER_AGENT_PREFIX, crate::VERSION));

    if let Some(ref token) = config.token {
        if !token.is_empty() {
            request = request.header("Authorization", token.as_str());
        }
    }

    let resp = request.json(&params).send().await
        .map_err(|e| TigerError::Config(format!("token 刷新 HTTP 请求失败: {}", e)))?;
    let body = resp.bytes().await
        .map_err(|e| TigerError::Config(format!("读取 token 刷新响应失败: {}", e)))?;

    let json: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| TigerError::Config(format!("解析 token 响应失败: {}", e)))?;

    let code = json.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
    if code != 0 {
        let msg = json.get("message").and_then(|v| v.as_str()).unwrap_or("unknown");
        return Err(TigerError::Config(format!("token 刷新接口返回错误: code={}, message={}", code, msg)));
    }

    let token = json
        .get("data")
        .and_then(|d| d.get("token"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    if token.is_empty() {
        return Err(TigerError::Config("服务端返回空 token".to_string()));
    }

    Ok(token.to_string())
}

#[cfg(test)]
mod tests;
