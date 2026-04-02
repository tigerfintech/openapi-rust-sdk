//! 推送消息类型定义和数据结构
//!
//! 定义 WebSocket 推送消息的类型、主题和数据结构，
//! 使用 JSON 序列化/反序列化（简化替代 Protobuf）。

use serde::{Deserialize, Serialize};

/// 推送消息类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    // 连接相关
    #[serde(rename = "connect")]
    Connect,
    #[serde(rename = "connected")]
    Connected,
    #[serde(rename = "disconnect")]
    Disconnect,
    #[serde(rename = "heartbeat")]
    Heartbeat,
    #[serde(rename = "kickout")]
    Kickout,
    #[serde(rename = "error")]
    Error,
    // 订阅相关
    #[serde(rename = "subscribe")]
    Subscribe,
    #[serde(rename = "unsubscribe")]
    Unsubscribe,
    // 行情推送
    #[serde(rename = "quote")]
    Quote,
    #[serde(rename = "tick")]
    Tick,
    #[serde(rename = "depth")]
    Depth,
    #[serde(rename = "option")]
    Option,
    #[serde(rename = "future")]
    Future,
    #[serde(rename = "kline")]
    Kline,
    #[serde(rename = "stock_top")]
    StockTop,
    #[serde(rename = "option_top")]
    OptionTop,
    #[serde(rename = "full_tick")]
    FullTick,
    #[serde(rename = "quote_bbo")]
    QuoteBbo,
    // 账户推送
    #[serde(rename = "asset")]
    Asset,
    #[serde(rename = "position")]
    Position,
    #[serde(rename = "order")]
    Order,
    #[serde(rename = "transaction")]
    Transaction,
}

/// 订阅主题类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubjectType {
    #[serde(rename = "quote")]
    Quote,
    #[serde(rename = "tick")]
    Tick,
    #[serde(rename = "depth")]
    Depth,
    #[serde(rename = "option")]
    Option,
    #[serde(rename = "future")]
    Future,
    #[serde(rename = "kline")]
    Kline,
    #[serde(rename = "stock_top")]
    StockTop,
    #[serde(rename = "option_top")]
    OptionTop,
    #[serde(rename = "full_tick")]
    FullTick,
    #[serde(rename = "quote_bbo")]
    QuoteBbo,
    #[serde(rename = "asset")]
    Asset,
    #[serde(rename = "position")]
    Position,
    #[serde(rename = "order")]
    Order,
    #[serde(rename = "transaction")]
    Transaction,
}

/// 推送消息通用结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushMessage {
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<SubjectType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// 连接认证请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectRequest {
    #[serde(rename = "tigerId")]
    pub tiger_id: String,
    pub sign: String,
    pub timestamp: String,
    pub version: String,
}

/// 订阅/退订请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeRequest {
    pub subject: SubjectType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
}

/// 行情推送数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuotePushData {
    pub symbol: String,
    #[serde(rename = "latestPrice", skip_serializing_if = "Option::is_none")]
    pub latest_price: Option<f64>,
    #[serde(rename = "preClose", skip_serializing_if = "Option::is_none")]
    pub pre_close: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

/// 逐笔成交推送数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TickPushData {
    pub symbol: String,
    pub price: f64,
    pub volume: i64,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub tick_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

/// 价格档位
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PriceLevel {
    pub price: f64,
    pub volume: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
}

/// 深度行情推送数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DepthPushData {
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asks: Option<Vec<PriceLevel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bids: Option<Vec<PriceLevel>>,
}

/// K 线推送数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KlinePushData {
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

/// 资产推送数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetPushData {
    pub account: String,
    #[serde(rename = "netLiquidation", skip_serializing_if = "Option::is_none")]
    pub net_liquidation: Option<f64>,
    #[serde(rename = "equityWithLoan", skip_serializing_if = "Option::is_none")]
    pub equity_with_loan: Option<f64>,
    #[serde(rename = "cashBalance", skip_serializing_if = "Option::is_none")]
    pub cash_balance: Option<f64>,
    #[serde(rename = "buyingPower", skip_serializing_if = "Option::is_none")]
    pub buying_power: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

/// 持仓推送数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionPushData {
    pub account: String,
    pub symbol: String,
    #[serde(rename = "secType", skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    pub quantity: i32,
    #[serde(rename = "averageCost", skip_serializing_if = "Option::is_none")]
    pub average_cost: Option<f64>,
    #[serde(rename = "marketPrice", skip_serializing_if = "Option::is_none")]
    pub market_price: Option<f64>,
    #[serde(rename = "marketValue", skip_serializing_if = "Option::is_none")]
    pub market_value: Option<f64>,
    #[serde(rename = "unrealizedPnl", skip_serializing_if = "Option::is_none")]
    pub unrealized_pnl: Option<f64>,
}

/// 订单推送数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderPushData {
    pub account: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(rename = "orderId", skip_serializing_if = "Option::is_none")]
    pub order_id: Option<i64>,
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(rename = "orderType", skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i32>,
    #[serde(rename = "limitPrice", skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filled: Option<i32>,
    #[serde(rename = "avgFillPrice", skip_serializing_if = "Option::is_none")]
    pub avg_fill_price: Option<f64>,
}

/// 成交推送数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionPushData {
    pub account: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(rename = "orderId", skip_serializing_if = "Option::is_none")]
    pub order_id: Option<i64>,
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}
