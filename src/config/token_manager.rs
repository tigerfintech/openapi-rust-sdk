//! TokenManager - Token 管理器
//!
//! 从 tiger_openapi_token.properties 文件加载 Token，
//! 支持后台定期刷新，更新内存和文件中的 Token。

use super::config_parser;
use crate::error::TigerError;
use std::sync::{Arc, RwLock};
use tokio::sync::oneshot;

/// 默认 Token 文件名
const DEFAULT_TOKEN_FILE: &str = "tiger_openapi_token.properties";
/// 默认后台检查间隔（秒）
const DEFAULT_CHECK_INTERVAL_SECS: u64 = 300; // 5 分钟

/// Token 管理器
pub struct TokenManager {
    token: Arc<RwLock<String>>,
    file_path: String,
    file_enabled: bool,
    /// Token 刷新阈值（秒），0 表示不刷新
    refresh_duration: i64,
    /// 后台检查间隔
    check_interval_secs: u64,
    /// Token 刷新后的回调（可选）
    token_writer: Option<Arc<dyn Fn(String) + Send + Sync>>,
    /// 自定义 Token 加载函数（可选），优先于文件加载
    token_loader: Option<Arc<dyn Fn() -> Result<String, TigerError> + Send + Sync>>,
    /// 停止信号发送端（Some 表示后台任务正在运行）
    stop_tx: Arc<RwLock<Option<oneshot::Sender<()>>>>,
}

impl TokenManager {
    /// 创建 Token 管理器（无文件持久化，仅内存）
    pub fn new() -> Self {
        Self::with_options(None, false, 0, DEFAULT_CHECK_INTERVAL_SECS, None, None)
    }

    /// 创建带 Token 文件路径的 Token 管理器（启用文件持久化）
    pub fn with_file_path(file_path: &str) -> Self {
        Self::with_options(
            Some(file_path),
            true,
            0,
            DEFAULT_CHECK_INTERVAL_SECS,
            None,
            None,
        )
    }

    /// 创建带刷新阈值的 Token 管理器
    /// refresh_duration_secs: 刷新阈值（秒），0 表示不刷新，最小 30 秒
    pub fn with_refresh_duration(file_path: Option<&str>, refresh_duration_secs: i64) -> Self {
        let dur = if refresh_duration_secs > 0 && refresh_duration_secs < 30 {
            30
        } else {
            refresh_duration_secs
        };
        Self::with_options(
            file_path,
            file_path.is_some(),
            dur,
            DEFAULT_CHECK_INTERVAL_SECS,
            None,
            None,
        )
    }

    fn with_options(
        file_path: Option<&str>,
        file_enabled: bool,
        refresh_duration: i64,
        check_interval_secs: u64,
        token_writer: Option<Arc<dyn Fn(String) + Send + Sync>>,
        token_loader: Option<Arc<dyn Fn() -> Result<String, TigerError> + Send + Sync>>,
    ) -> Self {
        Self {
            token: Arc::new(RwLock::new(String::new())),
            file_path: file_path.unwrap_or(DEFAULT_TOKEN_FILE).to_string(),
            file_enabled,
            refresh_duration,
            check_interval_secs,
            token_writer,
            token_loader,
            stop_tx: Arc::new(RwLock::new(None)),
        }
    }

    // ──────────────────────────────── builder setters ────────────────────────────────

    /// 设置 Token 刷新阈值（秒），最小 30 秒，0 = 禁用
    pub fn set_refresh_duration(&mut self, secs: i64) {
        self.refresh_duration = if secs > 0 && secs < 30 { 30 } else { secs };
    }

    /// 设置后台检查间隔（秒）
    pub fn set_check_interval(&mut self, secs: u64) {
        self.check_interval_secs = if secs == 0 {
            DEFAULT_CHECK_INTERVAL_SECS
        } else {
            secs
        };
    }

    /// 注册 token 刷新写入后的回调
    pub fn set_token_writer<F>(&mut self, writer: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.token_writer = Some(Arc::new(writer));
    }

    /// 注册自定义 token 加载函数（替代文件加载）
    pub fn set_token_loader<F>(&mut self, loader: F)
    where
        F: Fn() -> Result<String, TigerError> + Send + Sync + 'static,
    {
        self.token_loader = Some(Arc::new(loader));
    }

    // ──────────────────────────────── core API ────────────────────────────────

    /// 从 properties 文件（或自定义 loader）加载 Token
    pub fn load_token(&self) -> Result<String, TigerError> {
        if let Some(ref loader) = self.token_loader {
            let token = loader()
                .map_err(|e| TigerError::Config(format!("自定义 token 加载失败: {}", e)))?;
            if token.is_empty() {
                return Err(TigerError::Config("自定义 token 加载返回空值".to_string()));
            }
            *self.token.write().unwrap() = token.clone();
            return Ok(token);
        }

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

    /// 设置 Token 并写文件（如已启用文件持久化），成功后触发 token_writer 回调
    pub fn set_token(&self, token: &str) -> Result<(), TigerError> {
        *self.token.write().unwrap() = token.to_string();
        if self.file_enabled {
            self.save_token_to_file(token)?;
        }
        if let Some(ref writer) = self.token_writer {
            writer(token.to_string());
        }
        Ok(())
    }

    /// 仅更新内存中的 token，不写文件，不触发回调。
    /// 用于多个组件共享 token 时的内部同步（如 refresh_token 同步内部 TokenManager）。
    pub fn sync_token(&self, token: &str) {
        *self.token.write().unwrap() = token.to_string();
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

        let now_secs = i64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        )
        .unwrap_or(i64::MAX);

        (now_secs - gen_ts / 1000) > self.refresh_duration
    }

    /// 启动后台定期刷新任务。
    ///
    /// `refresh_fn` 是异步工厂函数，每次检查时若 `should_token_refresh()` 为 true 则调用，
    /// 返回新 token 后调用 `set_token`。
    ///
    /// 后台任务通过 tokio::spawn 运行，调用 `stop_auto_refresh` 停止。
    pub fn start_auto_refresh<F, Fut>(&self, refresh_fn: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<String, TigerError>> + Send + 'static,
    {
        // 停止之前的任务（如有）
        self.stop_auto_refresh();

        let (tx, rx) = oneshot::channel::<()>();
        *self.stop_tx.write().unwrap() = Some(tx);

        let token = Arc::clone(&self.token);
        let file_path = self.file_path.clone();
        let file_enabled = self.file_enabled;
        let refresh_duration = self.refresh_duration;
        let check_interval = std::time::Duration::from_secs(self.check_interval_secs);
        let token_writer = self.token_writer.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);
            let mut rx = rx;
            loop {
                tokio::select! {
                    _ = &mut rx => {
                        // stop signal received
                        return;
                    }
                    _ = interval.tick() => {
                        // Check if refresh is needed
                        let needs_refresh = {
                            let t = token.read().unwrap().clone();
                            if t.is_empty() || refresh_duration == 0 {
                                false
                            } else {
                                should_refresh_token(&t, refresh_duration)
                            }
                        };
                        if !needs_refresh {
                            continue;
                        }
                        match refresh_fn().await {
                            Ok(new_token) if !new_token.is_empty() => {
                                *token.write().unwrap() = new_token.clone();
                                if file_enabled {
                                    let _ = write_token_to_file(&file_path, &new_token);
                                }
                                if let Some(ref writer) = token_writer {
                                    writer(new_token);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
    }

    /// 停止后台刷新任务
    pub fn stop_auto_refresh(&self) {
        let tx = self.stop_tx.write().unwrap().take();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }
    }

    /// 将 Token 保存到 properties 文件
    fn save_token_to_file(&self, token: &str) -> Result<(), TigerError> {
        write_token_to_file(&self.file_path, token)
    }
}

impl Default for TokenManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TokenManager {
    fn drop(&mut self) {
        self.stop_auto_refresh();
    }
}

// ──────────────────────────────── free helpers ────────────────────────────────

/// Stateless helper used both inside the struct and the spawned task.
fn should_refresh_token(token: &str, refresh_duration: i64) -> bool {
    use base64::Engine;
    let decoded = match base64::engine::general_purpose::STANDARD.decode(token) {
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
    let now_secs = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    )
    .unwrap_or(i64::MAX);
    (now_secs - gen_ts / 1000) > refresh_duration
}

fn write_token_to_file(file_path: &str, token: &str) -> Result<(), TigerError> {
    let path = std::path::Path::new(file_path);
    if let Some(dir) = path.parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir)
                .map_err(|e| TigerError::Config(format!("创建目录失败: {}", e)))?;
        }
    }
    let content = format!("token={}\n", token);
    std::fs::write(file_path, content)
        .map_err(|e| TigerError::Config(format!("写入 Token 文件失败: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_token() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_rust_token.properties");
        std::fs::write(&path, "token=test_token_123\n").unwrap();
        let m = TokenManager::with_file_path(path.to_str().unwrap());
        let token = m.load_token().unwrap();
        assert_eq!(token, "test_token_123");
        assert_eq!(m.get_token(), "test_token_123");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_token_file_not_found() {
        let m = TokenManager::with_file_path("/nonexistent/path");
        assert!(m.load_token().is_err());
    }

    #[test]
    fn test_load_token_no_field() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_rust_token_nofield.properties");
        std::fs::write(&path, "other_key=value\n").unwrap();
        let m = TokenManager::with_file_path(path.to_str().unwrap());
        assert!(m.load_token().is_err());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_token_via_custom_loader() {
        let m = TokenManager::new();
        let mut m = m;
        m.set_token_loader(|| Ok("loader_token".to_string()));
        let token = m.load_token().unwrap();
        assert_eq!(token, "loader_token");
        assert_eq!(m.get_token(), "loader_token");
    }

    #[test]
    fn test_set_token_writes_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_rust_token_set.properties");
        let m = TokenManager::with_file_path(path.to_str().unwrap());
        m.set_token("new_token_456").unwrap();
        assert_eq!(m.get_token(), "new_token_456");
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "token=new_token_456\n");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_set_token_no_file_when_file_disabled() {
        let m = TokenManager::new(); // file_enabled = false
        m.set_token("mem_only_token").unwrap();
        assert_eq!(m.get_token(), "mem_only_token");
        // No file should exist at default path for this test
    }

    #[test]
    fn test_set_token_triggers_writer() {
        use std::sync::{Arc, Mutex};
        let captured: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let captured_clone = Arc::clone(&captured);
        let mut m = TokenManager::new();
        m.set_token_writer(move |t| {
            *captured_clone.lock().unwrap() = t;
        });
        m.set_token("callback_token").unwrap();
        assert_eq!(*captured.lock().unwrap(), "callback_token");
    }

    #[test]
    fn test_sync_token_no_file_no_callback() {
        use std::sync::{Arc, Mutex};
        let called: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
        let called_clone = Arc::clone(&called);
        let dir = std::env::temp_dir();
        let path = dir.join("test_rust_sync_token.properties");
        let mut m = TokenManager::with_file_path(path.to_str().unwrap());
        m.set_token_writer(move |_| {
            *called_clone.lock().unwrap() = true;
        });
        m.sync_token("sync_only");
        assert_eq!(m.get_token(), "sync_only");
        // file should NOT be written
        assert!(!path.exists());
        // callback should NOT be triggered
        assert!(!*called.lock().unwrap());
    }

    #[test]
    fn test_set_then_load() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_rust_token_roundtrip.properties");
        let m1 = TokenManager::with_file_path(path.to_str().unwrap());
        m1.set_token("round_trip_token").unwrap();
        let m2 = TokenManager::with_file_path(path.to_str().unwrap());
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
        assert!(!m.should_token_refresh());
    }

    #[test]
    fn test_should_token_refresh_zero_duration() {
        let m = TokenManager::new();
        m.set_token("some_token").ok();
        assert!(!m.should_token_refresh());
    }

    #[test]
    fn test_should_token_refresh_expired() {
        let m = TokenManager::with_refresh_duration(None, 30);
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
        let m = TokenManager::with_refresh_duration(None, 10);
        assert_eq!(m.refresh_duration, 30);
    }

    #[tokio::test]
    async fn test_start_auto_refresh_triggers_refresh() {
        use std::sync::{Arc, Mutex};

        let call_count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let call_count_clone = Arc::clone(&call_count);

        // Build an expired token (100 seconds old)
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let old_gen_ts = (now_secs - 100) * 1000;
        let expired_token = make_test_token(old_gen_ts, old_gen_ts + 3600000);

        let mut m = TokenManager::with_refresh_duration(None, 30);
        m.set_check_interval(0); // will be forced to default, use a short one
                                 // Override to 50ms for test speed
        m.check_interval_secs = 0; // we'll set it properly below
        m.check_interval_secs = 1; // 1 second interval for test
        m.set_token(&expired_token).unwrap();

        let updated_token: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let updated_token_clone = Arc::clone(&updated_token);
        m.set_token_writer(move |t| {
            *updated_token_clone.lock().unwrap() = t;
        });

        m.start_auto_refresh(move || {
            let count = Arc::clone(&call_count_clone);
            async move {
                *count.lock().unwrap() += 1;
                Ok("refreshed_token".to_string())
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        m.stop_auto_refresh();

        assert!(
            *call_count.lock().unwrap() > 0,
            "refresh should have been called"
        );
        assert_eq!(m.get_token(), "refreshed_token");
        assert_eq!(*updated_token.lock().unwrap(), "refreshed_token");
    }

    #[tokio::test]
    async fn test_stop_auto_refresh_stops_background_task() {
        use std::sync::{Arc, Mutex};
        let call_count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let old_gen_ts = (now_secs - 100) * 1000;
        let expired_token = make_test_token(old_gen_ts, old_gen_ts + 3600000);

        let mut m = TokenManager::with_refresh_duration(None, 30);
        m.check_interval_secs = 1;
        m.set_token(&expired_token).unwrap();
        m.start_auto_refresh(move || {
            let count = Arc::clone(&call_count_clone);
            async move {
                *count.lock().unwrap() += 1;
                Ok("new_token".to_string())
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        m.stop_auto_refresh();
        let count_after_stop = *call_count.lock().unwrap();

        // Wait more time; count should not increase after stop
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        let count_final = *call_count.lock().unwrap();
        assert_eq!(
            count_after_stop, count_final,
            "no more refreshes after stop"
        );
    }
}
