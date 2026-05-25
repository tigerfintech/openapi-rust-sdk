//! ClientConfig builder.
//!
//! Priority: environment variables > builder setters (incl. properties file) > auto-discovered config file > defaults.
//! Required fields (tiger_id, private_key) return TigerError::Config when empty.

use std::time::Duration;
use crate::error::TigerError;
use crate::model::enums::Language;
use crate::config::config_parser;
use crate::config::domain;

/// Default timeout in seconds
const DEFAULT_TIMEOUT_SECS: u64 = 15;
/// Default server URL
const DEFAULT_SERVER_URL: &str = "https://openapi.tigerfintech.com/gateway";

/// Tiger public key for response signature verification
const TIGER_PUBLIC_KEY: &str = "MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDNF3G8SoEcCZh2rshUbayDgLLrj6rKgzNMxDL2HSnKcB0+GPOsndqSv+a4IBu9+I3fyBp5hkyMMG2+AXugd9pMpy6VxJxlNjhX1MYbNTZJUT4nudki4uh+LMOkIBHOceGNXjgB+cXqmlUnjlqha/HgboeHSnSgpM3dKSJQlIOsDwIDAQAB";

/// Config file name for auto-discovery
const CONFIG_FILE_NAME: &str = "tiger_openapi_config.properties";

/// Environment variable names
const ENV_TIGER_ID: &str = "TIGEROPEN_TIGER_ID";
const ENV_PRIVATE_KEY: &str = "TIGEROPEN_PRIVATE_KEY";
const ENV_ACCOUNT: &str = "TIGEROPEN_ACCOUNT";
const ENV_TOKEN: &str = "TIGEROPEN_TOKEN";
const ENV_TOKEN_FILE: &str = "TIGEROPEN_TOKEN_FILE";

/// Client configuration
#[derive(Clone)]
pub struct ClientConfig {
    pub tiger_id: String,
    pub private_key: String,
    pub account: String,
    pub license: Option<String>,
    pub language: Language,
    pub timezone: Option<String>,
    pub timeout: Duration,
    pub token: Option<String>,
    /// Token 刷新阈值，超过此时长触发刷新，None/0 = 禁用
    pub token_refresh_duration: Option<Duration>,
    /// 后台 token 检查间隔，默认 5 分钟，仅 token_refresh_duration > 0 时生效
    pub token_check_interval: Option<Duration>,
    /// token 刷新成功后的可选回调
    #[allow(clippy::type_complexity)]
    pub token_writer: Option<std::sync::Arc<dyn Fn(String) + Send + Sync>>,
    /// 自定义 token 加载函数，替代默认文件加载
    #[allow(clippy::type_complexity)]
    pub token_loader: Option<std::sync::Arc<dyn Fn() -> Result<String, crate::error::TigerError> + Send + Sync>>,
    pub server_url: String,
    pub quote_server_url: String,
    pub tiger_public_key: String,
    pub device_id: String,
}

impl std::fmt::Debug for ClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientConfig")
            .field("tiger_id", &self.tiger_id)
            .field("private_key", &"[redacted]")
            .field("account", &self.account)
            .field("license", &self.license)
            .field("language", &self.language)
            .field("timezone", &self.timezone)
            .field("timeout", &self.timeout)
            .field("token", &self.token.as_deref().map(|_| "[set]"))
            .field("token_refresh_duration", &self.token_refresh_duration)
            .field("token_check_interval", &self.token_check_interval)
            .field("token_writer", &self.token_writer.as_ref().map(|_| "<fn>"))
            .field("token_loader", &self.token_loader.as_ref().map(|_| "<fn>"))
            .field("server_url", &self.server_url)
            .field("quote_server_url", &self.quote_server_url)
            .field("tiger_public_key", &"[redacted]")
            .field("device_id", &self.device_id)
            .finish()
    }
}

/// ClientConfig builder
pub struct ClientConfigBuilder {
    tiger_id: Option<String>,
    private_key: Option<String>,
    account: Option<String>,
    license: Option<String>,
    language: Option<Language>,
    timezone: Option<String>,
    timeout: Option<Duration>,
    token: Option<String>,
    token_refresh_duration: Option<Duration>,
    token_check_interval: Option<Duration>,
    #[allow(clippy::type_complexity)]
    token_writer: Option<std::sync::Arc<dyn Fn(String) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    token_loader: Option<std::sync::Arc<dyn Fn() -> Result<String, crate::error::TigerError> + Send + Sync>>,
    server_url: Option<String>,
    quote_server_url: Option<String>,
    enable_dynamic_domain: bool,
    tiger_public_key: Option<String>,
    device_id: Option<String>,
    /// When true, skip auto-discovery of config files (set when properties_file() is called)
    skip_auto_discover: bool,
}

impl ClientConfig {
    /// Create a new builder
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::new()
    }
}

impl ClientConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            tiger_id: None,
            private_key: None,
            account: None,
            license: None,
            language: None,
            timezone: None,
            timeout: None,
            token: None,
            token_refresh_duration: None,
            token_check_interval: None,
            token_writer: None,
            token_loader: None,
            server_url: None,
            quote_server_url: None,
            enable_dynamic_domain: true, // enabled by default
            tiger_public_key: None,
            device_id: None,
            skip_auto_discover: false,
        }
    }

    /// Set developer ID
    pub fn tiger_id(mut self, id: impl Into<String>) -> Self {
        self.tiger_id = Some(id.into());
        self
    }

    /// Set RSA private key
    pub fn private_key(mut self, key: impl Into<String>) -> Self {
        self.private_key = Some(key.into());
        self
    }

    /// Set trading account
    pub fn account(mut self, account: impl Into<String>) -> Self {
        self.account = Some(account.into());
        self
    }

    /// Set license type
    pub fn license(mut self, license: impl Into<String>) -> Self {
        self.license = Some(license.into());
        self
    }

    /// Set language
    pub fn language(mut self, lang: Language) -> Self {
        self.language = Some(lang);
        self
    }

    /// Set timezone
    pub fn timezone(mut self, tz: impl Into<String>) -> Self {
        self.timezone = Some(tz.into());
        self
    }

    /// Set request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set whether to enable dynamic domain resolution (enabled by default)
    pub fn enable_dynamic_domain(mut self, enable: bool) -> Self {
        self.enable_dynamic_domain = enable;
        self
    }

    /// Set TBHK license token
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Set token refresh threshold (triggers refresh when token age exceeds this)
    pub fn token_refresh_duration(mut self, d: Duration) -> Self {
        self.token_refresh_duration = Some(d);
        self
    }

    /// Set background token check interval (default 5 minutes)
    pub fn token_check_interval(mut self, d: Duration) -> Self {
        self.token_check_interval = Some(d);
        self
    }

    /// Set token refresh callback (called after every successful token write)
    pub fn token_writer<F>(mut self, writer: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.token_writer = Some(std::sync::Arc::new(writer));
        self
    }

    /// Set custom token loader (replaces default file loading)
    pub fn token_loader<F>(mut self, loader: F) -> Self
    where
        F: Fn() -> Result<String, crate::error::TigerError> + Send + Sync + 'static,
    {
        self.token_loader = Some(std::sync::Arc::new(loader));
        self
    }

    /// Set tiger public key for response signature verification
    pub fn tiger_public_key(mut self, key: impl Into<String>) -> Self {
        self.tiger_public_key = Some(key.into());
        self
    }

    /// Set quote server URL (separate gateway for quote requests)
    pub fn quote_server_url(mut self, url: impl Into<String>) -> Self {
        self.quote_server_url = Some(url.into());
        self
    }

    /// Set device ID (MAC address). Auto-detected if not set.
    pub fn device_id(mut self, id: impl Into<String>) -> Self {
        self.device_id = Some(id.into());
        self
    }

    /// Load config from a properties file.
    /// Silently skips if the file cannot be read; validation will catch missing required fields.
    pub fn properties_file(mut self, path: &str) -> Self {
        self.skip_auto_discover = true; // explicit file provided, skip auto-discovery
        if let Ok(props) = config_parser::parse_properties_file(path) {
            self.apply_properties(&props);
        }
        self
    }

    /// Apply properties key-value pairs to the builder (only fills unset fields)
    fn apply_properties(&mut self, props: &std::collections::HashMap<String, String>) {
        if self.tiger_id.is_none() {
            if let Some(v) = props.get("tiger_id") {
                self.tiger_id = Some(v.clone());
            }
        }
        // Private key priority: private_key > private_key_pk8 > private_key_pk1
        if self.private_key.is_none() {
            if let Some(v) = props.get("private_key") {
                self.private_key = Some(v.clone());
            } else if let Some(v) = props.get("private_key_pk8") {
                self.private_key = Some(v.clone());
            } else if let Some(v) = props.get("private_key_pk1") {
                self.private_key = Some(v.clone());
            }
        }
        if self.account.is_none() {
            if let Some(v) = props.get("account") {
                self.account = Some(v.clone());
            }
        }
        if self.license.is_none() {
            if let Some(v) = props.get("license") {
                self.license = Some(v.clone());
            }
        }
        if self.language.is_none() {
            if let Some(v) = props.get("language") {
                match v.as_str() {
                    "zh_CN" => self.language = Some(Language::ZhCn),
                    "zh_TW" => self.language = Some(Language::ZhTw),
                    "en_US" => self.language = Some(Language::EnUs),
                    _ => {}
                }
            }
        }
        if self.timezone.is_none() {
            if let Some(v) = props.get("timezone") {
                self.timezone = Some(v.clone());
            }
        }
    }

    /// Return candidate paths for auto-discovery of the config properties file.
    /// Search order: ./tiger_openapi_config.properties -> ~/.tigeropen/tiger_openapi_config.properties
    fn auto_discover_paths() -> Vec<String> {
        let mut paths = Vec::new();

        // 1. Current directory
        paths.push(format!("./{}", CONFIG_FILE_NAME));

        // 2. ~/.tigeropen/
        if let Ok(home) = std::env::var("HOME") {
            paths.push(format!("{}/.tigeropen/{}", home, CONFIG_FILE_NAME));
        }

        paths
    }

    /// Build ClientConfig.
    ///
    /// Resolution order: environment variables > builder setters (incl. properties file) > auto-discovered config > defaults.
    /// Returns TigerError::Config when required fields tiger_id or private_key are empty.
    pub fn build(mut self) -> Result<ClientConfig, TigerError> {
        // Auto-discover config file if no explicit values have been set for required fields.
        // Search order: ./tiger_openapi_config.properties -> ~/.tigeropen/tiger_openapi_config.properties
        // Skip if properties_file() was explicitly called (even if the file was not found).
        if !self.skip_auto_discover && (self.tiger_id.is_none() || self.private_key.is_none()) {
            let candidates = Self::auto_discover_paths();
            for path in &candidates {
                if let Ok(props) = config_parser::parse_properties_file(path) {
                    self.apply_properties(&props);
                    break; // use the first file found
                }
            }
        }

        // Environment variable overrides (highest priority)
        if let Ok(v) = std::env::var(ENV_TIGER_ID) {
            if !v.is_empty() {
                self.tiger_id = Some(v);
            }
        }
        if let Ok(v) = std::env::var(ENV_PRIVATE_KEY) {
            if !v.is_empty() {
                self.private_key = Some(v);
            }
        }
        if let Ok(v) = std::env::var(ENV_ACCOUNT) {
            if !v.is_empty() {
                self.account = Some(v);
            }
        }
        // Token loading priority: TIGEROPEN_TOKEN env > token_loader > token file (TIGEROPEN_TOKEN_FILE or default)
        if self.token.is_none() {
            if let Ok(v) = std::env::var(ENV_TOKEN) {
                if !v.is_empty() {
                    self.token = Some(v);
                }
            }
        }
        if self.token.is_none() {
            if let Some(ref loader) = self.token_loader {
                if let Ok(t) = loader() {
                    if !t.is_empty() {
                        self.token = Some(t);
                    }
                }
            }
        }
        if self.token.is_none() {
            let token_file_path = std::env::var(ENV_TOKEN_FILE)
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "tiger_openapi_token.properties".to_string());
            let tm = super::token_manager::TokenManager::with_file_path(&token_file_path);
            if let Ok(t) = tm.load_token() {
                if !t.is_empty() {
                    self.token = Some(t);
                }
            }
        }

        // Determine server URL: dynamic domain > default
        let (server_url, quote_server_url) = if let Some(url) = self.server_url {
            let quote_url = self.quote_server_url.unwrap_or_else(|| url.clone());
            (url, quote_url)
        } else {
            // Try dynamic domain resolution
            let mut resolved_server = String::new();
            let mut resolved_quote = String::new();
            if self.enable_dynamic_domain {
                let domain_conf = domain::query_domains(self.license.as_deref());
                if let Some(url) = domain::resolve_dynamic_server_url(&domain_conf, self.license.as_deref()) {
                    resolved_server = url;
                }
                if let Some(url) = domain::resolve_dynamic_quote_server_url(&domain_conf, self.license.as_deref()) {
                    resolved_quote = url;
                }
            }
            let server = if resolved_server.is_empty() {
                DEFAULT_SERVER_URL.to_string()
            } else {
                resolved_server
            };
            let quote = if let Some(url) = self.quote_server_url {
                url
            } else if resolved_quote.is_empty() {
                server.clone()
            } else {
                resolved_quote
            };
            (server, quote)
        };

        // Validate required fields
        let tiger_id = self.tiger_id.filter(|s| !s.is_empty()).ok_or_else(|| {
            TigerError::Config(format!(
                "tiger_id is required. Set it via builder().tiger_id(), env var {}, or a properties file",
                ENV_TIGER_ID
            ))
        })?;

        let private_key = self.private_key.filter(|s| !s.is_empty()).ok_or_else(|| {
            TigerError::Config(format!(
                "private_key is required. Set it via builder().private_key(), env var {}, or a properties file",
                ENV_PRIVATE_KEY
            ))
        })?;

        // Auto-detect device ID from MAC address if not explicitly set
        let device_id = self.device_id.unwrap_or_else(detect_device_id);

        Ok(ClientConfig {
            tiger_id,
            private_key,
            account: self.account.unwrap_or_default(),
            license: self.license,
            language: self.language.unwrap_or(Language::ZhCn),
            timezone: self.timezone,
            timeout: self.timeout.unwrap_or(Duration::from_secs(DEFAULT_TIMEOUT_SECS)),
            token: self.token,
            token_refresh_duration: self.token_refresh_duration,
            token_check_interval: self.token_check_interval,
            token_writer: self.token_writer,
            token_loader: self.token_loader,
            server_url,
            quote_server_url,
            tiger_public_key: self.tiger_public_key.unwrap_or_else(|| TIGER_PUBLIC_KEY.to_string()),
            device_id,
        })
    }
}

/// Auto-detect device ID from MAC address.
/// Returns the MAC address as a string (e.g. "AA:BB:CC:DD:EE:FF"), or empty string on failure.
fn detect_device_id() -> String {
    match mac_address::get_mac_address() {
        Ok(Some(ma)) => ma.to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Mutex;

    // 全局锁，确保环境变量测试串行执行
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    /// Acquire the env mutex, recovering from poison if a previous test panicked.
    fn lock_env() -> std::sync::MutexGuard<'static, ()> {
        ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// 清理环境变量的辅助函数
    fn clear_env_vars() {
        std::env::remove_var(ENV_TIGER_ID);
        std::env::remove_var(ENV_PRIVATE_KEY);
        std::env::remove_var(ENV_ACCOUNT);
        std::env::remove_var(ENV_TOKEN);
    }

    // ========== 单元测试 ==========

    #[test]
    fn test_builder_basic_fields() {
        let _lock = lock_env();
        clear_env_vars();
        let config = ClientConfig::builder()
            .tiger_id("test_id")
            .private_key("test_key")
            .account("DU123456")
            .build()
            .unwrap();
        assert_eq!(config.tiger_id, "test_id");
        assert_eq!(config.private_key, "test_key");
        assert_eq!(config.account, "DU123456");
    }

    #[test]
    fn test_builder_defaults() {
        let _lock = lock_env();
        clear_env_vars();
        let config = ClientConfig::builder()
            .tiger_id("test_id")
            .private_key("test_key")
            .build()
            .unwrap();
        assert_eq!(config.language, Language::ZhCn);
        assert_eq!(config.timeout, Duration::from_secs(15));
        assert_eq!(config.server_url, DEFAULT_SERVER_URL);
        assert_eq!(config.tiger_public_key, TIGER_PUBLIC_KEY);
    }

    #[test]
    fn test_builder_missing_tiger_id() {
        let _lock = lock_env();
        clear_env_vars();
        // Set a non-existent properties file to prevent auto-discovery from filling in tiger_id
        let result = ClientConfig::builder()
            .properties_file("/nonexistent/path/config.properties")
            .private_key("test_key")
            .build();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TigerError::Config(_)));
    }

    #[test]
    fn test_builder_missing_private_key() {
        let _lock = lock_env();
        clear_env_vars();
        // Set a non-existent properties file to prevent auto-discovery from filling in private_key
        let result = ClientConfig::builder()
            .properties_file("/nonexistent/path/config.properties")
            .tiger_id("test_id")
            .build();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TigerError::Config(_)));
    }

    #[test]
    fn test_env_overrides_builder() {
        let _lock = lock_env();
        clear_env_vars();
        std::env::set_var(ENV_TIGER_ID, "env_tiger_id");
        std::env::set_var(ENV_PRIVATE_KEY, "env_private_key");
        std::env::set_var(ENV_ACCOUNT, "env_account");
        let config = ClientConfig::builder()
            .tiger_id("builder_tiger_id")
            .private_key("builder_private_key")
            .account("builder_account")
            .build()
            .unwrap();
        assert_eq!(config.tiger_id, "env_tiger_id");
        assert_eq!(config.private_key, "env_private_key");
        assert_eq!(config.account, "env_account");
        clear_env_vars();
    }

    #[test]
    fn test_builder_optional_fields() {
        let _lock = lock_env();
        clear_env_vars();
        let config = ClientConfig::builder()
            .tiger_id("test_id")
            .private_key("test_key")
            .license("TBNZ")
            .language(Language::EnUs)
            .timezone("America/New_York")
            .timeout(Duration::from_secs(30))
            .token("my_token")
            .token_refresh_duration(Duration::from_secs(3600))
            .build()
            .unwrap();
        assert_eq!(config.license, Some("TBNZ".to_string()));
        assert_eq!(config.language, Language::EnUs);
        assert_eq!(config.timezone, Some("America/New_York".to_string()));
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.token, Some("my_token".to_string()));
        assert_eq!(config.token_refresh_duration, Some(Duration::from_secs(3600)));
    }

    #[test]
    fn test_builder_from_properties_file() {
        let _lock = lock_env();
        clear_env_vars();
        let dir = std::env::temp_dir();
        let path = dir.join("test_rust_client_config.properties");
        std::fs::write(
            &path,
            "tiger_id=file_tiger_id\nprivate_key=file_private_key\naccount=file_account\nlicense=TBHK\n",
        ).unwrap();
        let config = ClientConfig::builder()
            .properties_file(path.to_str().unwrap())
            .build()
            .unwrap();
        assert_eq!(config.tiger_id, "file_tiger_id");
        assert_eq!(config.private_key, "file_private_key");
        assert_eq!(config.account, "file_account");
        assert_eq!(config.license, Some("TBHK".to_string()));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_env_only_overrides_when_set() {
        let _lock = lock_env();
        clear_env_vars();
        std::env::set_var(ENV_TIGER_ID, "env_tiger_id");
        let config = ClientConfig::builder()
            .tiger_id("builder_tiger_id")
            .private_key("builder_private_key")
            .account("builder_account")
            .build()
            .unwrap();
        assert_eq!(config.tiger_id, "env_tiger_id");
        assert_eq!(config.private_key, "builder_private_key");
        assert_eq!(config.account, "builder_account");
        clear_env_vars();
    }

    // ========== Property 2 属性测试 ==========

    fn non_empty_string() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9_]{1,30}"
    }

    fn valid_timeout_secs() -> impl Strategy<Value = u64> {
        1u64..300u64
    }

    // **Validates: Requirements 2.1, 2.6**
    //
    // Feature: multi-language-sdks, Property 2: ClientConfig 字段设置 round-trip
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        #[test]
        fn client_config_field_round_trip(
            tiger_id in non_empty_string(),
            private_key in non_empty_string(),
            account in non_empty_string(),
            timeout_secs in valid_timeout_secs(),
        ) {
            let _lock = lock_env();
            clear_env_vars();
            let config = ClientConfig::builder()
                .tiger_id(&tiger_id)
                .private_key(&private_key)
                .account(&account)
                .timeout(Duration::from_secs(timeout_secs))
                .build()
                .unwrap();
            prop_assert_eq!(&config.tiger_id, &tiger_id);
            prop_assert_eq!(&config.private_key, &private_key);
            prop_assert_eq!(&config.account, &account);
            prop_assert_eq!(config.timeout, Duration::from_secs(timeout_secs));
        }
    }

    // ========== Property 3 属性测试 ==========

    // **Validates: Requirements 2.4**
    //
    // Feature: multi-language-sdks, Property 3: 环境变量优先级高于配置文件
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        #[test]
        fn env_overrides_builder_values(
            env_tiger_id in non_empty_string(),
            env_private_key in non_empty_string(),
            env_account in non_empty_string(),
            builder_tiger_id in non_empty_string(),
            builder_private_key in non_empty_string(),
            builder_account in non_empty_string(),
        ) {
            let _lock = lock_env();
            clear_env_vars();
            std::env::set_var(ENV_TIGER_ID, &env_tiger_id);
            std::env::set_var(ENV_PRIVATE_KEY, &env_private_key);
            std::env::set_var(ENV_ACCOUNT, &env_account);
            let config = ClientConfig::builder()
                .tiger_id(&builder_tiger_id)
                .private_key(&builder_private_key)
                .account(&builder_account)
                .build()
                .unwrap();
            prop_assert_eq!(&config.tiger_id, &env_tiger_id);
            prop_assert_eq!(&config.private_key, &env_private_key);
            prop_assert_eq!(&config.account, &env_account);
            clear_env_vars();
        }
    }
}
