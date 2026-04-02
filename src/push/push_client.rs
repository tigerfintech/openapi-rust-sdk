//! PushClient - WebSocket 推送客户端
//!
//! 通过 WebSocket 长连接接收实时行情和账户推送。
//! 支持连接认证、订阅/退订、回调机制、心跳保活和自动重连。
//! 注意：实际 WebSocket 连接需要运行时环境，此处提供完整的
//! 订阅状态管理和消息处理逻辑，连接部分需要 tokio 运行时。

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::config::ClientConfig;
use super::callbacks::Callbacks;
use super::push_message::*;

/// 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

/// PushClient 配置选项
#[derive(Default)]
pub struct PushClientOptions {
    pub push_url: Option<String>,
    pub heartbeat_interval_secs: Option<u64>,
    pub reconnect_interval_secs: Option<u64>,
    pub auto_reconnect: Option<bool>,
}

/// WebSocket 推送客户端
pub struct PushClient {
    config: ClientConfig,
    push_url: String,
    auto_reconnect: bool,
    state: Arc<RwLock<ConnectionState>>,
    callbacks: Arc<RwLock<Callbacks>>,
    /// 行情订阅状态：subject -> symbols set
    subscriptions: Arc<RwLock<HashMap<SubjectType, HashSet<String>>>>,
    /// 账户级别订阅
    account_subs: Arc<RwLock<HashSet<SubjectType>>>,
    /// 发送消息的通道
    tx: Arc<RwLock<Option<tokio::sync::mpsc::UnboundedSender<String>>>>,
}

impl PushClient {
    /// 创建推送客户端
    pub fn new(config: ClientConfig, options: Option<PushClientOptions>) -> Self {
        let opts = options.unwrap_or_default();
        Self {
            config,
            push_url: opts.push_url.unwrap_or_else(|| "wss://openapi-push.tigerfintech.com".into()),
            auto_reconnect: opts.auto_reconnect.unwrap_or(true),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            callbacks: Arc::new(RwLock::new(Callbacks::default())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            account_subs: Arc::new(RwLock::new(HashSet::new())),
            tx: Arc::new(RwLock::new(None)),
        }
    }

    /// 获取当前连接状态
    pub fn state(&self) -> ConnectionState {
        *self.state.read().unwrap()
    }

    /// 设置回调函数集合
    pub fn set_callbacks(&self, cb: Callbacks) {
        *self.callbacks.write().unwrap() = cb;
    }

    /// 断开连接
    pub fn disconnect(&self) {
        *self.state.write().unwrap() = ConnectionState::Disconnected;
        *self.tx.write().unwrap() = None;
        let cbs = self.callbacks.read().unwrap();
        if let Some(cb) = &cbs.on_disconnect {
            cb();
        }
    }

    /// 处理收到的消息
    pub fn handle_message(&self, raw: &str) {
        let msg: PushMessage = match serde_json::from_str(raw) {
            Ok(m) => m,
            Err(_) => {
                let cbs = self.callbacks.read().unwrap();
                if let Some(cb) = &cbs.on_error {
                    cb("反序列化消息失败".to_string());
                }
                return;
            }
        };
        let cbs = self.callbacks.read().unwrap();
        match msg.msg_type {
            MessageType::Kickout => {
                if let (Some(cb), Some(data)) = (&cbs.on_kickout, &msg.data) {
                    if let Some(s) = data.as_str() {
                        cb(s.to_string());
                    }
                }
            }
            MessageType::Error => {
                if let (Some(cb), Some(data)) = (&cbs.on_error, &msg.data) {
                    cb(format!("服务端错误: {}", data));
                }
            }
            MessageType::Quote => {
                if let (Some(cb), Some(data)) = (&cbs.on_quote, &msg.data) {
                    if let Ok(d) = serde_json::from_value(data.clone()) {
                        cb(d);
                    }
                }
            }
            MessageType::Tick => {
                if let (Some(cb), Some(data)) = (&cbs.on_tick, &msg.data) {
                    if let Ok(d) = serde_json::from_value(data.clone()) {
                        cb(d);
                    }
                }
            }
            MessageType::Depth => {
                if let (Some(cb), Some(data)) = (&cbs.on_depth, &msg.data) {
                    if let Ok(d) = serde_json::from_value(data.clone()) {
                        cb(d);
                    }
                }
            }
            MessageType::Option => {
                if let (Some(cb), Some(data)) = (&cbs.on_option, &msg.data) {
                    if let Ok(d) = serde_json::from_value(data.clone()) { cb(d); }
                }
            }
            MessageType::Future => {
                if let (Some(cb), Some(data)) = (&cbs.on_future, &msg.data) {
                    if let Ok(d) = serde_json::from_value(data.clone()) { cb(d); }
                }
            }
            MessageType::Kline => {
                if let (Some(cb), Some(data)) = (&cbs.on_kline, &msg.data) {
                    if let Ok(d) = serde_json::from_value(data.clone()) { cb(d); }
                }
            }
            MessageType::Asset => {
                if let (Some(cb), Some(data)) = (&cbs.on_asset, &msg.data) {
                    if let Ok(d) = serde_json::from_value(data.clone()) { cb(d); }
                }
            }
            MessageType::Position => {
                if let (Some(cb), Some(data)) = (&cbs.on_position, &msg.data) {
                    if let Ok(d) = serde_json::from_value(data.clone()) { cb(d); }
                }
            }
            MessageType::Order => {
                if let (Some(cb), Some(data)) = (&cbs.on_order, &msg.data) {
                    if let Ok(d) = serde_json::from_value(data.clone()) { cb(d); }
                }
            }
            MessageType::Transaction => {
                if let (Some(cb), Some(data)) = (&cbs.on_transaction, &msg.data) {
                    if let Ok(d) = serde_json::from_value(data.clone()) { cb(d); }
                }
            }
            MessageType::StockTop => {
                if let (Some(cb), Some(data)) = (&cbs.on_stock_top, &msg.data) {
                    if let Ok(d) = serde_json::from_value(data.clone()) { cb(d); }
                }
            }
            MessageType::OptionTop => {
                if let (Some(cb), Some(data)) = (&cbs.on_option_top, &msg.data) {
                    if let Ok(d) = serde_json::from_value(data.clone()) { cb(d); }
                }
            }
            MessageType::FullTick => {
                if let (Some(cb), Some(data)) = (&cbs.on_full_tick, &msg.data) {
                    if let Ok(d) = serde_json::from_value(data.clone()) { cb(d); }
                }
            }
            MessageType::QuoteBbo => {
                if let (Some(cb), Some(data)) = (&cbs.on_quote_bbo, &msg.data) {
                    if let Ok(d) = serde_json::from_value(data.clone()) { cb(d); }
                }
            }
            _ => {}
        }
    }

    // ===== 订阅状态管理 =====

    /// 添加订阅记录
    pub fn add_subscription(&self, subject: SubjectType, symbols: &[String]) {
        let mut subs = self.subscriptions.write().unwrap();
        let set = subs.entry(subject).or_insert_with(HashSet::new);
        for s in symbols {
            set.insert(s.clone());
        }
    }

    /// 移除订阅记录
    pub fn remove_subscription(&self, subject: SubjectType, symbols: Option<&[String]>) {
        let mut subs = self.subscriptions.write().unwrap();
        match symbols {
            None => { subs.remove(&subject); }
            Some(syms) => {
                if let Some(set) = subs.get_mut(&subject) {
                    for s in syms { set.remove(s); }
                    if set.is_empty() { subs.remove(&subject); }
                }
            }
        }
    }

    /// 获取当前行情订阅状态
    pub fn get_subscriptions(&self) -> HashMap<SubjectType, Vec<String>> {
        let subs = self.subscriptions.read().unwrap();
        subs.iter().map(|(k, v)| {
            (k.clone(), v.iter().cloned().collect())
        }).collect()
    }

    /// 添加账户订阅
    pub fn add_account_sub(&self, subject: SubjectType) {
        self.account_subs.write().unwrap().insert(subject);
    }

    /// 移除账户订阅
    pub fn remove_account_sub(&self, subject: &SubjectType) {
        self.account_subs.write().unwrap().remove(subject);
    }

    /// 获取账户级别订阅状态
    pub fn get_account_subscriptions(&self) -> Vec<SubjectType> {
        self.account_subs.read().unwrap().iter().cloned().collect()
    }
}
