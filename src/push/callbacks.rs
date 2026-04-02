//! 推送回调函数类型定义
//!
//! 每种推送类型独立回调，连接状态回调独立管理。

use super::push_message::*;
use std::sync::Arc;

/// 所有回调函数的集合
#[derive(Default, Clone)]
pub struct Callbacks {
    // 行情推送回调
    pub on_quote: Option<Arc<dyn Fn(QuotePushData) + Send + Sync>>,
    pub on_tick: Option<Arc<dyn Fn(TickPushData) + Send + Sync>>,
    pub on_depth: Option<Arc<dyn Fn(DepthPushData) + Send + Sync>>,
    pub on_option: Option<Arc<dyn Fn(QuotePushData) + Send + Sync>>,
    pub on_future: Option<Arc<dyn Fn(QuotePushData) + Send + Sync>>,
    pub on_kline: Option<Arc<dyn Fn(KlinePushData) + Send + Sync>>,
    pub on_stock_top: Option<Arc<dyn Fn(QuotePushData) + Send + Sync>>,
    pub on_option_top: Option<Arc<dyn Fn(QuotePushData) + Send + Sync>>,
    pub on_full_tick: Option<Arc<dyn Fn(TickPushData) + Send + Sync>>,
    pub on_quote_bbo: Option<Arc<dyn Fn(QuotePushData) + Send + Sync>>,
    // 账户推送回调
    pub on_asset: Option<Arc<dyn Fn(AssetPushData) + Send + Sync>>,
    pub on_position: Option<Arc<dyn Fn(PositionPushData) + Send + Sync>>,
    pub on_order: Option<Arc<dyn Fn(OrderPushData) + Send + Sync>>,
    pub on_transaction: Option<Arc<dyn Fn(TransactionPushData) + Send + Sync>>,
    // 连接状态回调
    pub on_connect: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_disconnect: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_error: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub on_kickout: Option<Arc<dyn Fn(String) + Send + Sync>>,
}
