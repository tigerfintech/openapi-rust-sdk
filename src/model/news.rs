//! 资讯响应模型。
//!
//! - 响应结构体使用 `#[serde(rename_all = "camelCase")]` 匹配服务端返回（决策 #13：出参驼峰）。
//! - `NewsItem` / `NewsPage` 为 `news_stock_list` 与 `news_highlight_list` 两接口共用
//!   （服务端共用同一套 VO）。

use serde::{Deserialize, Serialize};

// ============================================================================
// 资讯条目与分页信封（两接口共用）
// ============================================================================

/// 单条资讯（news_stock_list / news_highlight_list 共用）。
///
/// 服务端出参为 camelCase，字段名与 JSON key 相同。
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewsItem {
    /// 文章 ID（highlight 游标翻页 `last_id` 的取值来源）
    #[serde(default)]
    pub id: String,
    /// 标题
    #[serde(default)]
    pub title: String,
    /// 摘要
    #[serde(default)]
    pub summary: String,
    /// 发布时间戳（秒级）
    #[serde(default)]
    pub pub_timestamp: i64,
    /// 详情页 URL
    #[serde(default)]
    pub url: String,
}

/// 资讯列表分页信封（news_stock_list / news_highlight_list 共用）。
///
/// - `news_stock_list` 按 `page_count` / `page_size` 翻页；
/// - `news_highlight_list` 只按 `last_id` 游标翻页，响应中 `page_count` 恒为 1。
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewsPage {
    /// 当前页资讯列表（服务端返回空列表时为空数组，不报错）
    #[serde(default)]
    pub items: Vec<NewsItem>,
    /// 当前页码（highlight 接口恒为 1）
    #[serde(default)]
    pub page_count: i32,
    /// 每页条数
    #[serde(default)]
    pub page_size: i32,
    /// 总页数
    #[serde(default)]
    pub total_page: i32,
    /// 总条数
    #[serde(default)]
    pub total_size: i32,
}
