//! HttpClient 封装 HTTP 请求、签名、重试、超时。

use std::collections::BTreeMap;

use crate::config::client_config::ClientConfig;
use crate::error::TigerError;
use crate::signer::{get_sign_content, sign_with_rsa};
use super::api_request::ApiRequest;
use super::api_response::{ApiResponse, parse_api_response};
use super::retry::RetryPolicy;

/// User-Agent 前缀
const USER_AGENT_PREFIX: &str = "openapi-rust-sdk-";
/// SDK 版本号
const SDK_VERSION: &str = "0.1.0";
/// 默认字符集
const DEFAULT_CHARSET: &str = "UTF-8";
/// 默认签名类型
const DEFAULT_SIGN_TYPE: &str = "RSA";
/// 默认 API 版本
const DEFAULT_VERSION: &str = "2.0";

/// HttpClient 封装 HTTP 请求、签名、重试、超时
pub struct HttpClient {
    config: ClientConfig,
    client: reqwest::Client,
    retry_policy: RetryPolicy,
}

impl HttpClient {
    /// 创建 HttpClient 实例
    pub fn new(config: ClientConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap_or_default();
        Self {
            config,
            client,
            retry_policy: RetryPolicy::default(),
        }
    }

    /// 使用自定义 reqwest::Client 创建 HttpClient（用于测试）
    pub fn with_client(config: ClientConfig, client: reqwest::Client) -> Self {
        Self {
            config,
            client,
            retry_policy: RetryPolicy::default(),
        }
    }

    /// 获取 User-Agent 字符串
    pub fn user_agent() -> String {
        format!("{}{}", USER_AGENT_PREFIX, SDK_VERSION)
    }

    /// 构造公共请求参数
    fn build_common_params(&self, api_method: &str, biz_content: &str) -> BTreeMap<String, String> {
        let mut params = BTreeMap::new();
        params.insert("tiger_id".to_string(), self.config.tiger_id.clone());
        params.insert("method".to_string(), api_method.to_string());
        params.insert("charset".to_string(), DEFAULT_CHARSET.to_string());
        params.insert("sign_type".to_string(), DEFAULT_SIGN_TYPE.to_string());
        params.insert("timestamp".to_string(), chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        params.insert("version".to_string(), DEFAULT_VERSION.to_string());
        params.insert("biz_content".to_string(), biz_content.to_string());
        params
    }

    /// 对参数进行签名
    fn sign_params(&self, params: &BTreeMap<String, String>) -> Result<String, TigerError> {
        let content = get_sign_content(params);
        sign_with_rsa(&self.config.private_key, &content)
    }

    /// 执行结构化 API 请求，返回解析后的 ApiResponse
    pub async fn execute_request(&self, request: &ApiRequest) -> Result<ApiResponse, TigerError> {
        let mut params = self.build_common_params(&request.method, &request.biz_content);
        let sign = self.sign_params(&params)?;
        params.insert("sign".to_string(), sign);

        let max_attempts = if self.retry_policy.should_retry(&request.method) {
            self.retry_policy.max_retries + 1
        } else {
            1
        };

        let mut last_err = None;
        for attempt in 0..max_attempts {
            if attempt > 0 {
                let backoff = self.retry_policy.calculate_backoff(attempt - 1);
                tokio::time::sleep(backoff).await;
            }

            match self.do_http_post(&params).await {
                Ok(body) => {
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

    /// 通用 API 调用方法
    /// api_method: API 方法名（如 "market_state"、"place_order"）
    /// request_json: 原始 biz_content JSON 字符串
    /// 返回原始 response JSON 字符串，不做任何解析
    pub async fn execute(&self, api_method: &str, request_json: &str) -> Result<String, TigerError> {
        // 参数校验
        if api_method.is_empty() {
            return Err(TigerError::Config("api_method 不能为空".to_string()));
        }
        // 校验 request_json 是否为有效 JSON
        if serde_json::from_str::<serde_json::Value>(request_json).is_err() {
            return Err(TigerError::Config("request_json 不是有效的 JSON".to_string()));
        }

        let mut params = self.build_common_params(api_method, request_json);
        let sign = self.sign_params(&params)?;
        params.insert("sign".to_string(), sign);

        let max_attempts = if self.retry_policy.should_retry(api_method) {
            self.retry_policy.max_retries + 1
        } else {
            1
        };

        let mut last_err = None;
        for attempt in 0..max_attempts {
            if attempt > 0 {
                let backoff = self.retry_policy.calculate_backoff(attempt - 1);
                tokio::time::sleep(backoff).await;
            }

            match self.do_http_post(&params).await {
                Ok(body) => {
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

    /// 发送 HTTP POST 请求
    async fn do_http_post(&self, params: &BTreeMap<String, String>) -> Result<Vec<u8>, TigerError> {
        let mut request = self.client
            .post(&self.config.server_url)
            .header("Content-Type", "application/json;charset=UTF-8")
            .header("User-Agent", Self::user_agent());

        if let Some(ref token) = self.config.token {
            if !token.is_empty() {
                request = request.header("Authorization", token.as_str());
            }
        }

        let resp = request
            .json(params)
            .send()
            .await?;

        let body = resp.bytes().await?;
        Ok(body.to_vec())
    }
}

#[cfg(test)]
mod tests;
