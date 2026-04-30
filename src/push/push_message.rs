//! 推送消息类型定义
//!
//! 定义订阅主题类型枚举。
//! JSON 消息类型已移除，使用 Protobuf 生成的 `pb::*` 类型替代。

/// 订阅主题类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubjectType {
    Quote,
    Tick,
    Depth,
    Option,
    Future,
    Kline,
    StockTop,
    OptionTop,
    FullTick,
    QuoteBbo,
    Asset,
    Position,
    Order,
    Transaction,
}
