//! 资讯请求结构体。
//!
//! 所有字段使用 snake_case（与 wire 协议直接对齐，决策 #13：入参 snake_case），
//! 可选字段 `Option<T>` + `skip_serializing_if` 跳过空值——缺省语义由服务端归一化：
//! `page_count` 缺省 = 1，`page_size` 缺省 = 20。

use crate::model::enums::Language;
use serde::Serialize;

/// 个股新闻列表请求。wire: news_stock_list
///
/// - `symbol` 必填、单标的、非空：下游资讯服务实际不支持批量查询（传多个仅单个标的生效，
///   决策 #5），空值由服务端校验报参数错误；
/// - `page_count` / `page_size` 可选：缺省（含不落 key）由服务端归一化为 1 / 20；
///   显式传值须满足 `page_count >= 1`、`1 <= page_size <= 50`（越界由服务端报参数错误）；
/// - `lang` 可选：`zh_CN` / `zh_TW` / `en_US`（`Language` 枚举）。
#[derive(Debug, Clone, Serialize, Default)]
pub struct NewsStockListRequest {
    /// 必填。单标的：下游资讯接口实际不支持批量查询
    pub symbol: String,
    /// 页码。缺省 = 服务端默认 1；显式传值须 >= 1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_count: Option<i32>,
    /// 每页条数。缺省 = 服务端默认 20；显式传值须 1-50（闭区间）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
    /// 语言枚举（zh_CN / zh_TW / en_US）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<Language>,
}

/// 市场要闻列表请求。wire: news_highlight_list
///
/// 无必填参数。翻页只走 `last_id` 游标（决策 #14：该接口无 page_count / page_size）：
/// 取上一页 `items` 最后一条的 `id` 作为 `last_id` 获取下一页；
/// 不传则从最新开始。服务端固定按每页一游标返回，响应 `page_count` 恒为 1。
#[derive(Debug, Clone, Serialize, Default)]
pub struct NewsHighlightListRequest {
    /// 游标翻页参数：上一页 items 最后一条的 id；None = 从最新开始
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    /// 语言枚举（zh_CN / zh_TW / en_US）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<Language>,
}
