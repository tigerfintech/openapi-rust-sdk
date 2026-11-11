//! 资讯客户端，封装资讯相关 API（`news_stock_list` / `news_highlight_list`）。
//!
//! 请求走统一网关 `POST /gateway`（method + biz_content，与其余 HTTP 接口同一签名链路），
//! 并路由到**主网关** `config.server_url`（交易域同款），不走行情集群
//! `config.quote_server_url`——资讯域流量不得打进行情集群。
//!
//! 所有方法返回强类型响应（[`NewsPage`]），
//! 请求参数使用 snake_case 序列化（匹配服务端约定）。

use serde::Serialize;
use serde_json::Value;

use crate::client::api_request::ApiRequest;
use crate::client::decode::decode_value;
use crate::client::http_client::HttpClient;
use crate::config::client_config::ClientConfig;
use crate::error::TigerError;
use crate::model::news::NewsPage;
use crate::model::news_requests::{NewsHighlightListRequest, NewsStockListRequest};

/// 资讯客户端。
///
/// 路由约定：[`NewsClient::from_config`] 内部使用 [`HttpClient::new`]（主网关
/// `config.server_url`），与 [`crate::trade::TradeClient`] 同款；**不要**用
/// [`HttpClient::with_quote_server`] 构造（那会把资讯流量发往行情集群）。
pub struct NewsClient {
    http_client: HttpClient,
}

impl NewsClient {
    /// 从 ClientConfig 自动构造（推荐）。请求发往主网关 `config.server_url`。
    pub fn from_config(config: ClientConfig) -> Self {
        Self {
            http_client: HttpClient::new(config),
        }
    }

    /// 使用已有的 HttpClient 创建资讯客户端。
    ///
    /// 传入的 HttpClient 应指向主网关（即经 [`HttpClient::new`] 构造）；
    /// 经 [`HttpClient::with_quote_server`] 构造的客户端会把请求发往行情域名。
    pub fn new(http_client: HttpClient) -> Self {
        Self { http_client }
    }

    /// 停止后台 token 自动刷新任务（若在运行）。
    /// 客户端不再使用时调用。
    pub fn close(&self) {
        self.http_client.close();
    }

    /// 内部通用：构造请求、发送、把 data 解析为 [`NewsPage`]。
    async fn call_news<P>(&self, method: &str, params: P) -> Result<NewsPage, TigerError>
    where
        P: Serialize,
    {
        let biz = serde_json::to_string(&params)
            .map_err(|e| TigerError::Config(format!("serialize biz params failed: {}", e)))?;
        let req = ApiRequest::new(method, biz);
        let resp = self.http_client.execute_request(&req).await?;
        unmarshal_news_page(resp.data)
    }

    /// 查询个股新闻列表。wire: news_stock_list
    ///
    /// - `symbol` 必填、**单标的**（下游资讯服务不支持批量查询，传多个仅单个标的生效）；
    /// - 翻页：`page_count` / `page_size`（缺省由服务端归一化为 1 / 20；
    ///   显式传值须 `page_count >= 1`、`1 <= page_size <= 50`，越界由服务端报参数错误）；
    /// - `lang` 可选（`zh_CN` / `zh_TW` / `en_US`）。
    pub async fn get_stock_news(&self, req: NewsStockListRequest) -> Result<NewsPage, TigerError> {
        self.call_news("news_stock_list", req).await
    }

    /// 查询市场要闻列表。wire: news_highlight_list
    ///
    /// 无必填参数。**翻页只走 `last_id` 游标**（该接口无 page_count / page_size）：
    /// 取上一页 `items` 最后一条的 `id` 作为下一次调用的 `last_id`；
    /// `None` 则从最新开始。响应的 `page_count` 恒为 1。
    pub async fn get_market_news(
        &self,
        req: NewsHighlightListRequest,
    ) -> Result<NewsPage, TigerError> {
        self.call_news("news_highlight_list", req).await
    }
}

/// 将响应 `data` 解析为 [`NewsPage`]。
///
/// `data` 缺失 / null 时返回空页（对齐 quote 侧 `unmarshal_data` 惯例），
/// 双重编码的 JSON 字符串由 [`decode_value`] 解一层。
fn unmarshal_news_page(data: Option<Value>) -> Result<NewsPage, TigerError> {
    match data {
        None => Ok(NewsPage::default()),
        Some(v) if v.is_null() => Ok(NewsPage::default()),
        Some(v) => decode_value(v),
    }
}

#[cfg(test)]
mod tests;
