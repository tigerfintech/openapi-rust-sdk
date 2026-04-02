//! TokenManager - Token 管理器
//!
//! 从 tiger_openapi_token.properties 文件加载 Token，
//! 支持后台定期刷新，更新内存和文件中的 Token。

use std::sync::{Arc, RwLock};
use crate::error::TigerError;
use super::config_parser;

/// 默认 Token 文件名
const DEFAULT_TOKEN_FILE: &str = "tiger_openapi_token.properties";

/// Token 管理器
pub struct TokenManager {
    token: Arc<RwLock<String>>,
    file_path: String,
    /// Token 刷新阈值（秒），0 表示不刷新
    refresh_duration: i64,
}

impl TokenManager {
    /// 创建 Token 管理器
    pub fn new(file_path: Option<&str>) -> Self {
        Self {
            token: Arc::new(RwLock::new(String::new())),
            file_path: file_path.unwrap_or(DEFAULT_TOKEN_FILE).to_string(),
            refresh_duration: 0,
        }
    }

    /// 创建带刷新阈值的 Token 管理器
    /// refresh_duration_secs: 刷新阈值（秒），0 表示不刷新，最小 30 秒
    pub fn with_refresh_duration(file_path: Option<&str>, refresh_duration_secs: i64) -> Self {
        let dur = if refresh_duration_secs > 0 && refresh_duration_secs < 30 {
            30
        } else {
            refresh_duration_secs
        };
        Self {
            token: Arc::new(RwLock::new(String::new())),
            file_path: file_path.unwrap_or(DEFAULT_TOKEN_FILE).to_string(),
            refresh_duration: dur,
        }
    }

    /// 从 properties 文件加载 Token
    pub fn load_token(&self) -> Result<String, TigerError> {
        let props = config_parser::parse_properties_file(&self.file_path)?;
        let token = props.get("token").cloned().unwrap_or_default();
        if token.is_empty() {
            return Err(TigerError::Config(
                "Token 文件中未找到 token 字段".to_string(),
            ));
        }
        *self.token.write().unwrap() = token.clone();
        Ok(token)
    }

    /// 获取当前 Token
    pub fn get_token(&self) -> String {
        self.token.read().unwrap().clone()
    }

    /// 设置 Token 并更新文件
    pub fn set_token(&self, token: &str) -> Result<(), TigerError> {
        *self.token.write().unwrap() = token.to_string();
        self.save_token_to_file(token)
    }

    /// 判断 Token 是否需要刷新。
    /// 解码 base64 token，提取前 27 字符中的 gen_ts，
    /// 当 (当前时间秒 - gen_ts/1000) > refresh_duration 时返回 true。
    pub fn should_token_refresh(&self) -> bool {
        let token = self.token.read().unwrap().clone();
        if token.is_empty() || self.refresh_duration == 0 {
            return false;
        }

        use base64::Engine;
        let decoded = match base64::engine::general_purpose::STANDARD.decode(&token) {
            Ok(d) => d,
            Err(_) => return false,
        };
        if decoded.len() < 27 {
            return false;
        }

        let header = match std::str::from_utf8(&decoded[..27]) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let parts: Vec<&str> = header.splitn(2, ',').collect();
        if parts.len() < 2 {
            return false;
        }

        let gen_ts: i64 = match parts[0].trim().parse() {
            Ok(v) => v,
            Err(_) => return false,
        };

        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        (now_secs - gen_ts / 1000) > self.refresh_duration
    }

    /// 将 Token 保存到 properties 文件
    fn save_token_to_file(&self, token: &str) -> Result<(), TigerError> {
        let path = std::path::Path::new(&self.file_path);
        if let Some(dir) = path.parent() {
            if !dir.as_os_str().is_empty() {
                std::fs::create_dir_all(dir).map_err(|e| {
                    TigerError::Config(format!("创建目录失败: {}", e))
                })?;
            }
        }
        let content = format!("token={}\n", token);
        std::fs::write(&self.file_path, content).map_err(|e| {
            TigerError::Config(format!("写入 Token 文件失败: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_token() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_rust_token.properties");
        std::fs::write(&path, "token=test_token_123\n").unwrap();
        let m = TokenManager::new(Some(path.to_str().unwrap()));
        let token = m.load_token().unwrap();
        assert_eq!(token, "test_token_123");
        assert_eq!(m.get_token(), "test_token_123");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_token_file_not_found() {
        let m = TokenManager::new(Some("/nonexistent/path"));
        assert!(m.load_token().is_err());
    }

    #[test]
    fn test_load_token_no_field() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_rust_token_nofield.properties");
        std::fs::write(&path, "other_key=value\n").unwrap();
        let m = TokenManager::new(Some(path.to_str().unwrap()));
        assert!(m.load_token().is_err());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_set_token() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_rust_token_set.properties");
        let m = TokenManager::new(Some(path.to_str().unwrap()));
        m.set_token("new_token_456").unwrap();
        assert_eq!(m.get_token(), "new_token_456");
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "token=new_token_456\n");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_set_then_load() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_rust_token_roundtrip.properties");
        let m1 = TokenManager::new(Some(path.to_str().unwrap()));
        m1.set_token("round_trip_token").unwrap();
        let m2 = TokenManager::new(Some(path.to_str().unwrap()));
        let token = m2.load_token().unwrap();
        assert_eq!(token, "round_trip_token");
        std::fs::remove_file(&path).ok();
    }

    /// 构造测试用 base64 token，前 27 字符包含 "gen_ts_ms,expire_ts_ms"
    fn make_test_token(gen_ts_ms: i64, expire_ts_ms: i64) -> String {
        use base64::Engine;
        let header = format!("{:013},{:013}", gen_ts_ms, expire_ts_ms);
        let payload = format!("{}some_extra_payload_data", header);
        base64::engine::general_purpose::STANDARD.encode(payload.as_bytes())
    }

    #[test]
    fn test_should_token_refresh_empty_token() {
        let m = TokenManager::with_refresh_duration(None, 30);
        // 空 token 不需要刷新
        assert!(!m.should_token_refresh());
    }

    #[test]
    fn test_should_token_refresh_zero_duration() {
        let m = TokenManager::new(None);
        m.set_token("some_token").ok();
        // refresh_duration 为 0 时不刷新
        assert!(!m.should_token_refresh());
    }

    #[test]
    fn test_should_token_refresh_expired() {
        let m = TokenManager::with_refresh_duration(None, 30);
        // gen_ts 在 100 秒前
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let old_gen_ts = (now_secs - 100) * 1000;
        let token = make_test_token(old_gen_ts, old_gen_ts + 3600000);
        *m.token.write().unwrap() = token;
        assert!(m.should_token_refresh());
    }

    #[test]
    fn test_should_token_refresh_not_expired() {
        let m = TokenManager::with_refresh_duration(None, 3600);
        // gen_ts 刚刚生成
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let fresh_gen_ts = now_secs * 1000;
        let token = make_test_token(fresh_gen_ts, fresh_gen_ts + 7200000);
        *m.token.write().unwrap() = token;
        assert!(!m.should_token_refresh());
    }

    #[test]
    fn test_should_token_refresh_invalid_token() {
        let m = TokenManager::with_refresh_duration(None, 30);
        *m.token.write().unwrap() = "not_valid_base64!!!".to_string();
        assert!(!m.should_token_refresh());
    }

    #[test]
    fn test_with_refresh_duration_min_30() {
        // 小于 30 秒应被强制设为 30
        let m = TokenManager::with_refresh_duration(None, 10);
        assert_eq!(m.refresh_duration, 30);
    }
}
