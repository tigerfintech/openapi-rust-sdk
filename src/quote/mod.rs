//! 行情客户端，封装所有行情相关 API。
//!
//! 所有方法返回强类型响应（[`MarketState`]、[`Brief`]、[`Kline`] 等），
//! 请求参数使用 snake_case 序列化（匹配服务端约定）。

use serde::Serialize;
use serde_json::Value;

use crate::client::api_request::ApiRequest;
use crate::client::decode::decode_value;
use crate::client::http_client::HttpClient;
use crate::config::client_config::ClientConfig;
use crate::error::TigerError;
use crate::model::quote::{
    Brief, CapitalDistribution, CapitalFlow, CorporateAction, CorporateActionRequest, Depth,
    ExchangeRate, FinancialCurrency, FinancialDailyItem, FinancialDailyRequest,
    FinancialReportItem, FinancialReportRequest, FundContractInfo, FundHistoryQuote, FundQuote,
    FutureContractInfo, FutureDepth, FutureExchange, FutureKline,
    FutureMainContractHistory, FutureQuote, FutureTradingTime, FutureTradeTickItem,
    IndustryItem, IndustryStock, Kline, KlineItem, KlineQuota, MarketScannerRequest,
    MarketScannerTags, MarketState, OptionAnalysis, OptionBrief, OptionChain, OptionExpiration,
    OptionKline, OptionSymbol, QuoteOvernight, QuotePermission, ScannerResult, ShortInterest,
    StockBroker, StockDetail, StockIndustry, SymbolName, Timeline, TradeTick, TradeRankItem,
    TradingCalendarItem, WarrantBrief, WarrantFilterResult, FutureKlineItem,
};
use crate::model::quote_requests::{
    AllFutureContractsRequest, KlineRequest, KlineByPageRequest, BriefRequest, DepthQuoteRequest,
    FinancialCurrencyRequest, FinancialExchangeRateRequest, FutureKlineRequest,
    FutureKlineByPageRequest, FutureBriefRequest, FutureContinuousContractsRequest,
    FutureContractSingleRequest, FutureDepthRequest, FutureHistoryMainContractRequest,
    FutureTradingTimesRequest, FutureTradeTicksRequest, FundContractsRequest, FundHistoryQuoteRequest,
    FundQuoteRequest, FundSymbolsRequest, IndustryListRequest, IndustryStocksRequest,
    KlineQuotaRequest, MarketScannerTagsRequest, OptionAnalysisRequest,
    OptionDepthRequest, OptionSymbolsRequest, OptionTimelineRequest, OptionTradeTicksRequest,
    OptionChainRequest, OptionContractItem, OptionKlineRequest, OptionQuoteRequest,
    QuoteOvernightRequest, QuotePermissionRequest, ShortInterestRequest, StockBrokerRequest,
    StockDelayBriefsRequest, StockDetailsRequest, StockFundamentalRequest, StockIndustryRequest,
    SymbolsRequest, TimelineHistoryRequest, TradeMetasRequest, TradeRankRequest, TradeTickRequest,
    TradingCalendarRequest, WarrantBriefsRequest, WarrantFilterRequest,
};

/// API 版本常量
const VERSION_V1: &str = "1.0";
const VERSION_V2: &str = "2.0";
const VERSION_V3: &str = "3.0";

/// 行情客户端，封装所有行情 API。
pub struct QuoteClient {
    http_client: HttpClient,
}

impl QuoteClient {
    /// 从 ClientConfig 自动构造（推荐）。内部使用 quote server URL。
    pub fn from_config(config: ClientConfig) -> Self {
        Self { http_client: HttpClient::with_quote_server(config) }
    }

    /// 使用已有的 HttpClient 创建行情客户端。调用方通常使用 [`HttpClient::with_quote_server`]
    /// 以便请求发往 `config.quote_server_url`。
    pub fn new(http_client: HttpClient) -> Self {
        Self { http_client }
    }

    /// 内部通用：构造请求、发送、把 data 解析为 T。
    pub async fn call_into<T, P>(&self, method: &str, params: P) -> Result<T, TigerError>
    where
        T: serde::de::DeserializeOwned + Default,
        P: Serialize,
    {
        self.call_into_versioned(method, params, None).await
    }

    pub async fn call_into_versioned<T, P>(
        &self,
        method: &str,
        params: P,
        version: Option<&str>,
    ) -> Result<T, TigerError>
    where
        T: serde::de::DeserializeOwned + Default,
        P: Serialize,
    {
        let biz = serde_json::to_string(&params)
            .map_err(|e| TigerError::Config(format!("serialize biz params failed: {}", e)))?;
        let req = match version {
            Some(v) => ApiRequest::with_version(method, biz, v),
            None => ApiRequest::new(method, biz),
        };
        let resp = self.http_client.execute_request(&req).await?;
        unmarshal_data(resp.data)
    }

    /// 解包 {items:[...]} 外壳并返回 items 列表。
    pub async fn call_into_items<T, P>(&self, method: &str, params: P) -> Result<Vec<T>, TigerError>
    where
        T: serde::de::DeserializeOwned,
        P: Serialize,
    {
        let wrap: serde_json::Value = self.call_into(method, params).await?;
        match wrap.get("items") {
            None | Some(Value::Null) => Ok(vec![]),
            Some(items) => serde_json::from_value(items.clone())
                .map_err(|e| TigerError::Config(format!("decode items failed: {}", e))),
        }
    }

    /// 解包可能返回数组或单对象的接口（统一成 Vec）。
    pub async fn call_into_list_or_object<T, P>(
        &self,
        method: &str,
        params: P,
    ) -> Result<Vec<T>, TigerError>
    where
        T: serde::de::DeserializeOwned,
        P: Serialize,
    {
        let biz = serde_json::to_string(&params)
            .map_err(|e| TigerError::Config(format!("serialize biz params failed: {}", e)))?;
        let req = ApiRequest::new(method, biz);
        let resp = self.http_client.execute_request(&req).await?;
        let raw = match resp.data {
            None | Some(Value::Null) => return Ok(vec![]),
            Some(v) => v,
        };
        // 先尝试 list
        if let Ok(list) = serde_json::from_value::<Vec<T>>(raw.clone()) {
            return Ok(list);
        }
        // 若为字符串则先解一层
        let inner = if let Value::String(s) = &raw {
            serde_json::from_str::<Value>(s)
                .map_err(|e| TigerError::Config(format!("decode double-encoded data: {}", e)))?
        } else {
            raw
        };
        // 尝试 object -> 包装成 [object]
        let obj: T = serde_json::from_value(inner)
            .map_err(|e| TigerError::Config(format!("decode single object failed: {}", e)))?;
        Ok(vec![obj])
    }

    // ========== 基础行情 ==========

    /// 获取市场状态
    pub async fn get_market_state(&self, market: &str) -> Result<Vec<MarketState>, TigerError> {
        self.call_into("market_state", serde_json::json!({ "market": market })).await
    }

    /// 获取实时行情。wire: quote_real_time
    pub async fn get_real_time_quote(&self, req: BriefRequest) -> Result<Vec<Brief>, TigerError> {
        self.call_into("quote_real_time", req).await
    }

    #[deprecated(since = "0.5.1", note = "Use get_real_time_quote instead")]
    pub async fn get_brief(&self, req: BriefRequest) -> Result<Vec<Brief>, TigerError> {
        self.get_real_time_quote(req).await
    }

    /// 获取 K 线。wire: kline
    pub async fn get_kline(&self, req: KlineRequest) -> Result<Vec<Kline>, TigerError> {
        self.call_into("kline", req).await
    }

    /// 获取分时数据（v3.0）
    pub async fn get_timeline(&self, symbols: &[&str]) -> Result<Vec<Timeline>, TigerError> {
        self.call_into_versioned(
            "timeline",
            serde_json::json!({ "symbols": symbols }),
            Some(VERSION_V3),
        )
        .await
    }

    /// 获取逐笔成交（v0.4.0 新签名：接受 TradeTickRequest）。wire: trade_tick
    pub async fn get_trade_tick(&self, req: TradeTickRequest) -> Result<Vec<TradeTick>, TigerError> {
        self.call_into("trade_tick", req).await
    }

    /// 获取深度报价（v0.4.0 新签名：接受 DepthQuoteRequest）。wire: quote_depth
    pub async fn get_quote_depth(&self, req: DepthQuoteRequest) -> Result<Vec<Depth>, TigerError> {
        self.call_into("quote_depth", req).await
    }

    // ========== 股票基础查询 ==========

    /// 全量合约代码。wire: all_symbols（返回字符串列表）
    pub async fn get_symbols(&self, req: SymbolsRequest) -> Result<Vec<String>, TigerError> {
        self.call_into("all_symbols", req).await
    }

    /// 全量合约代码 + 名称。wire: all_symbol_names
    pub async fn get_symbol_names(&self, req: SymbolsRequest) -> Result<Vec<SymbolName>, TigerError> {
        self.call_into("all_symbol_names", req).await
    }

    /// 交易元数据。wire: quote_stock_trade
    pub async fn get_trade_metas(&self, req: TradeMetasRequest) -> Result<Vec<crate::model::quote::TradeMeta>, TigerError> {
        self.call_into("quote_stock_trade", req).await
    }

    /// 股票详情。wire: stock_detail（服务端返回 {items:[...]}）
    pub async fn get_stock_details(&self, req: StockDetailsRequest) -> Result<Vec<StockDetail>, TigerError> {
        self.call_into_items("stock_detail", req).await
    }

    /// 延时行情。wire: quote_delay
    pub async fn get_delayed_quote(&self, req: StockDelayBriefsRequest) -> Result<Vec<Brief>, TigerError> {
        self.call_into("quote_delay", req).await
    }

    #[deprecated(since = "0.5.1", note = "Use get_delayed_quote instead")]
    pub async fn get_stock_delay_briefs(&self, req: StockDelayBriefsRequest) -> Result<Vec<Brief>, TigerError> {
        self.get_delayed_quote(req).await
    }

    /// 客户端分页 K 线。循环调用直到获得 total_size 条。
    pub async fn get_kline_by_page(&self, req: KlineByPageRequest) -> Result<Vec<KlineItem>, TigerError> {
        let page_size = req.page_size.unwrap_or(200);
        let total_size = req.total_size.unwrap_or(1000);
        let mut acc: Vec<KlineItem> = Vec::new();
        let mut end_time = req.end_time;
        let begin_time = req.begin_time;

        while (acc.len() as i32) < total_size {
            let sub = KlineRequest {
                symbols: req.symbols.clone(),
                period: req.period.clone(),
                right: req.right.clone(),
                begin_time,
                end_time,
                limit: Some(page_size),
                trade_session: req.trade_session.clone(),
                lang: req.lang.clone(),
                ..Default::default()
            };
            let page_out: Vec<Kline> = self.call_into("kline", sub).await?;
            if page_out.is_empty() || page_out[0].items.is_empty() {
                break;
            }
            let items = page_out[0].items.clone();
            let len = items.len() as i32;
            acc.extend(items.clone());
            if len < page_size {
                break;
            }
            // 以最早 bar 的时间 - 1 作为下一页 end_time
            let oldest = items.iter().map(|i| i.time).min().unwrap_or(0);
            end_time = Some(oldest - 1);
        }
        Ok(acc)
    }

    /// 历史分时。wire: history_timeline
    pub async fn get_timeline_history(&self, req: TimelineHistoryRequest) -> Result<Vec<Timeline>, TigerError> {
        self.call_into("history_timeline", req).await
    }

    /// 成交榜单。wire: trade_rank
    pub async fn get_trade_rank(&self, req: TradeRankRequest) -> Result<Vec<TradeRankItem>, TigerError> {
        self.call_into("trade_rank", req).await
    }

    /// 做空数据。wire: quote_shortable_stocks
    pub async fn get_short_interest(&self, req: ShortInterestRequest) -> Result<Vec<ShortInterest>, TigerError> {
        self.call_into("quote_shortable_stocks", req).await
    }

    /// 经纪商持仓。wire: stock_broker
    pub async fn get_stock_broker(&self, req: StockBrokerRequest) -> Result<Option<StockBroker>, TigerError> {
        self.call_optional("stock_broker", req).await
    }

    /// 股票基本面。wire: stock_fundamental（返回 map，由调用方解析）
    pub async fn get_stock_fundamental(
        &self,
        req: StockFundamentalRequest,
    ) -> Result<std::collections::BTreeMap<String, serde_json::Value>, TigerError> {
        self.call_into("stock_fundamental", req).await
    }

    /// 股票行业归属。wire: stock_industry（返回数组）
    pub async fn get_stock_industry(&self, req: StockIndustryRequest) -> Result<Vec<StockIndustry>, TigerError> {
        self.call_into("stock_industry", req).await
    }

    /// 行情权限详情。wire: get_quote_permission
    pub async fn get_quote_permission(&self, req: QuotePermissionRequest) -> Result<Vec<QuotePermission>, TigerError> {
        self.call_into("get_quote_permission", req).await
    }

    /// K 线配额。wire: kline_quota
    pub async fn get_kline_quota(&self, req: KlineQuotaRequest) -> Result<Vec<KlineQuota>, TigerError> {
        self.call_into("kline_quota", req).await
    }

    // ========== 期权行情 ==========

    /// 获取期权到期日。HK 市场需传 `market = Some("HK")` 及 HK 标的代码（如 `"00700"`）。
    pub async fn get_option_expiration(&self, symbols: &[&str], market: Option<&str>) -> Result<Vec<OptionExpiration>, TigerError> {
        let mut payload = serde_json::json!({ "symbols": symbols });
        if let Some(m) = market {
            payload["market"] = serde_json::Value::String(m.to_string());
        }
        self.call_into("option_expiration", payload).await
    }

    /// 获取期权链（v3.0）。
    ///
    /// ```rust,ignore
    /// // 日期字符串，按 symbol 自动推断时区
    /// qc.get_option_chain(OptionChainRequest::new(vec![
    ///     OptionChainItem::from_date("AAPL", "2024-01-19")?,
    /// ])).await?;
    ///
    /// // 直接传毫秒时间戳
    /// qc.get_option_chain(OptionChainRequest::new(vec![
    ///     OptionChainItem::new("AAPL", 1705622400000),
    /// ])).await?;
    /// ```
    pub async fn get_option_chain(
        &self,
        req: OptionChainRequest,
    ) -> Result<Vec<OptionChain>, TigerError> {
        self.call_into_versioned("option_chain", req, Some(VERSION_V3)).await
    }

    /// 获取期权实时行情（v2.0）。
    ///
    /// ```rust,ignore
    /// // OCC 格式，按 symbol 自动推断时区
    /// qc.get_option_quote(OptionQuoteRequest::new(vec![
    ///     OptionContractItem::from_occ("AAPL 240119C00150000")?,
    /// ])).await?;
    ///
    /// // 直接传时间戳
    /// qc.get_option_quote(OptionQuoteRequest::new(vec![
    ///     OptionContractItem::new("AAPL", 1705622400000, "Call", "150.000"),
    /// ])).await?;
    /// ```
    pub async fn get_option_quote(
        &self,
        req: OptionQuoteRequest,
    ) -> Result<Vec<OptionBrief>, TigerError> {
        self.call_into_versioned("option_brief", req, Some(VERSION_V2)).await
    }

    #[deprecated(since = "0.5.1", note = "Use get_option_quote instead")]
    pub async fn get_option_brief(
        &self,
        identifiers: &[&str],
    ) -> Result<Vec<OptionBrief>, TigerError> {
        let items: Result<Vec<_>, _> = identifiers.iter()
            .map(|id| OptionContractItem::from_occ(id))
            .collect();
        self.get_option_quote(OptionQuoteRequest::new(items?)).await
    }

    /// 获取期权 K 线（v2.0）。
    ///
    /// ```rust,ignore
    /// // OCC 格式，按 symbol 自动推断时区
    /// qc.get_option_kline(OptionKlineRequest {
    ///     option_query: Some(vec![
    ///         OptionKlineItem::from_occ("AAPL 240119C00150000", "day")?,
    ///     ]),
    ///     ..Default::default()
    /// }).await?;
    /// ```
    pub async fn get_option_kline(
        &self,
        req: OptionKlineRequest,
    ) -> Result<Vec<OptionKline>, TigerError> {
        self.call_into_versioned("option_kline", req, Some(VERSION_V2)).await
    }

    /// 期权逐笔。wire: option_trade_tick
    pub async fn get_option_trade_ticks(&self, req: OptionTradeTicksRequest) -> Result<Vec<TradeTick>, TigerError> {
        self.call_into("option_trade_tick", req).await
    }

    /// 期权分时。wire: option_timeline
    pub async fn get_option_timeline(&self, req: OptionTimelineRequest) -> Result<Vec<Timeline>, TigerError> {
        self.call_into("option_timeline", req).await
    }

    /// 期权盘口。wire: option_depth
    pub async fn get_option_depth(&self, req: OptionDepthRequest) -> Result<Vec<Depth>, TigerError> {
        self.call_into("option_depth", req).await
    }

    /// 期权代码列表（港股）。wire: all_hk_option_symbols
    pub async fn get_option_symbols(&self, req: OptionSymbolsRequest) -> Result<Vec<OptionSymbol>, TigerError> {
        self.call_into("all_hk_option_symbols", req).await
    }

    /// 期权分析（隐含/历史波动率）。wire: option_analysis
    pub async fn get_option_analysis(&self, req: OptionAnalysisRequest) -> Result<Vec<OptionAnalysis>, TigerError> {
        self.call_into("option_analysis", req).await
    }

    // ========== 期货行情 ==========

    /// 获取期货交易所列表
    pub async fn get_future_exchange(&self) -> Result<Vec<FutureExchange>, TigerError> {
        self.call_into("future_exchange", serde_json::json!({ "sec_type": "FUT" })).await
    }

    /// 获取期货合约列表（by exchange code）
    pub async fn get_future_contracts(
        &self,
        exchange_code: &str,
    ) -> Result<Vec<FutureContractInfo>, TigerError> {
        self.call_into(
            "future_contract_by_exchange_code",
            serde_json::json!({ "exchange_code": exchange_code }),
        )
        .await
    }

    /// 获取期货实时报价（v0.4.0 新签名：接受 FutureBriefRequest）。wire: future_real_time_quote
    pub async fn get_future_real_time_quote(&self, req: FutureBriefRequest) -> Result<Vec<FutureQuote>, TigerError> {
        self.call_into("future_real_time_quote", req).await
    }

    /// 获取期货 K 线。wire: future_kline
    /// begin_time / end_time 为 0 时自动改为 -1（服务端要求字段必须存在）。
    pub async fn get_future_kline(&self, mut req: FutureKlineRequest) -> Result<Vec<FutureKline>, TigerError> {
        if req.begin_time == Some(0) {
            req.begin_time = Some(-1);
        }
        if req.end_time == Some(0) {
            req.end_time = Some(-1);
        }
        self.call_into("future_kline", req).await
    }

    /// 按 contract_code 查询单个期货合约。wire: future_contract_by_contract_code
    /// 服务端可能返回单对象或列表，统一展开为 Vec。
    pub async fn get_future_contract(&self, req: FutureContractSingleRequest) -> Result<Vec<FutureContractInfo>, TigerError> {
        self.call_into_list_or_object("future_contract_by_contract_code", req).await
    }

    /// 查询所有期货合约。wire: future_contracts
    pub async fn get_all_future_contracts(&self, req: AllFutureContractsRequest) -> Result<Vec<FutureContractInfo>, TigerError> {
        self.call_into("future_contracts", req).await
    }

    /// 当前主力合约。wire: future_current_contract
    pub async fn get_current_future_contract(&self, req: FutureContractSingleRequest) -> Result<Option<FutureContractInfo>, TigerError> {
        self.call_optional("future_current_contract", req).await
    }

    /// 连续主力合约。wire: future_continuous_contracts
    /// 服务端可能返回单对象或列表，统一展开为 Vec。
    pub async fn get_future_continuous_contracts(&self, req: FutureContinuousContractsRequest) -> Result<Vec<FutureContractInfo>, TigerError> {
        self.call_into_list_or_object("future_continuous_contracts", req).await
    }

    /// 主力合约历史。wire: future_main_contract
    pub async fn get_future_history_main_contract(&self, req: FutureHistoryMainContractRequest) -> Result<Vec<FutureMainContractHistory>, TigerError> {
        self.call_into("future_main_contract", req).await
    }

    /// 期货 K 线分页包装。
    pub async fn get_future_kline_by_page(&self, req: FutureKlineByPageRequest) -> Result<Vec<FutureKlineItem>, TigerError> {
        let page_size = req.page_size.unwrap_or(200);
        let total_size = req.total_size.unwrap_or(1000);
        let mut acc: Vec<FutureKlineItem> = Vec::new();
        let mut end_time = if req.end_time == Some(0) || req.end_time.is_none() {
            Some(-1i64)
        } else {
            req.end_time
        };
        let begin_time = if req.begin_time == Some(0) || req.begin_time.is_none() {
            Some(-1i64)
        } else {
            req.begin_time
        };

        while (acc.len() as i32) < total_size {
            let sub = FutureKlineRequest {
                contract_code: req.contract_code.clone(),
                period: req.period.clone(),
                begin_time,
                end_time,
                limit: Some(page_size),
                lang: req.lang.clone(),
                ..Default::default()
            };
            let page_out: Vec<FutureKline> = self.call_into("future_kline", sub).await?;
            if page_out.is_empty() || page_out[0].items.is_empty() {
                break;
            }
            let items = page_out[0].items.clone();
            let len = items.len() as i32;
            acc.extend(items.clone());
            if len < page_size {
                break;
            }
            let oldest = items.iter().map(|i| i.time).min().unwrap_or(0);
            end_time = Some(oldest - 1);
        }
        Ok(acc)
    }

    /// 期货逐笔。wire: future_tick (API version 3.0)
    /// 服务端返回 {contractCode, items:[...]} 对象，需解包 items 并回填 contractCode。
    pub async fn get_future_trade_ticks(&self, req: FutureTradeTicksRequest) -> Result<Vec<FutureTradeTickItem>, TigerError> {
        let mut req = req;
        // end_index 服务端要求 >= 0；未设置时默认 30（与 Python/Go SDK 一致）
        if req.end_index.is_none() {
            req.end_index = Some(30);
        }
        #[derive(serde::Deserialize, Default)]
        #[serde(rename_all = "camelCase")]
        struct FutureTickWrap {
            #[serde(default)]
            contract_code: String,
            #[serde(default)]
            items: Vec<FutureTradeTickItem>,
        }
        let wrap: FutureTickWrap = self.call_into_versioned("future_tick", req, Some(VERSION_V3)).await?;
        let mut items = wrap.items;
        for item in &mut items {
            if item.contract_code.is_empty() {
                item.contract_code = wrap.contract_code.clone();
            }
        }
        Ok(items)
    }

    /// 期货盘口。wire: future_depth
    pub async fn get_future_depth(&self, req: FutureDepthRequest) -> Result<Vec<FutureDepth>, TigerError> {
        self.call_into("future_depth", req).await
    }

    /// 期货交易时段。wire: future_trading_date（返回单个对象）
    pub async fn get_future_trading_times(&self, req: FutureTradingTimesRequest) -> Result<Option<FutureTradingTime>, TigerError> {
        self.call_optional("future_trading_date", req).await
    }

    // ========== 基金 ==========

    /// 基金代码列表。wire: fund_all_symbols（返回字符串列表）
    pub async fn get_fund_symbols(&self, req: FundSymbolsRequest) -> Result<Vec<String>, TigerError> {
        self.call_into("fund_all_symbols", req).await
    }

    /// 基金合约信息。wire: fund_contracts
    pub async fn get_fund_contracts(&self, req: FundContractsRequest) -> Result<Vec<FundContractInfo>, TigerError> {
        self.call_into("fund_contracts", req).await
    }

    /// 基金实时净值。wire: fund_quote
    pub async fn get_fund_quote(&self, req: FundQuoteRequest) -> Result<Vec<FundQuote>, TigerError> {
        self.call_into("fund_quote", req).await
    }

    /// 基金历史净值。wire: fund_history_quote
    pub async fn get_fund_history_quote(&self, req: FundHistoryQuoteRequest) -> Result<Vec<FundHistoryQuote>, TigerError> {
        self.call_into("fund_history_quote", req).await
    }

    // ========== 窝轮 ==========

    /// 窝轮实时行情。wire: warrant_briefs
    pub async fn get_warrant_quote(&self, req: WarrantBriefsRequest) -> Result<Vec<WarrantBrief>, TigerError> {
        self.call_into("warrant_briefs", req).await
    }

    #[deprecated(since = "0.5.1", note = "Use get_warrant_quote instead")]
    pub async fn get_warrant_briefs(&self, req: WarrantBriefsRequest) -> Result<Vec<WarrantBrief>, TigerError> {
        self.get_warrant_quote(req).await
    }

    /// 窝轮筛选。wire: warrant_filter
    pub async fn get_warrant_filter(&self, req: WarrantFilterRequest) -> Result<Option<WarrantFilterResult>, TigerError> {
        self.call_optional("warrant_filter", req).await
    }

    // ========== 行业 ==========

    /// 行业列表。wire: industry_list
    pub async fn get_industry_list(&self, req: IndustryListRequest) -> Result<Vec<IndustryItem>, TigerError> {
        self.call_into("industry_list", req).await
    }

    /// 行业下股票列表。wire: industry_stock_list
    pub async fn get_industry_stocks(&self, req: IndustryStocksRequest) -> Result<Vec<IndustryStock>, TigerError> {
        self.call_into("industry_stock_list", req).await
    }

    // ========== 公司行动 ==========

    /// 获取公司行动。服务端返回 `{symbol: [...]}` 结构，内部扁平化为单个数组。
    pub async fn get_corporate_action(
        &self,
        req: CorporateActionRequest,
    ) -> Result<Vec<CorporateAction>, TigerError> {
        let grouped: std::collections::BTreeMap<String, Vec<CorporateAction>> =
            self.call_into("corporate_action", req).await?;
        let mut out = Vec::new();
        for (_, mut list) in grouped {
            out.append(&mut list);
        }
        Ok(out)
    }

    /// 公司行动 - 拆股。wire: corporate_action (action_type=split)
    pub async fn get_corporate_split(
        &self,
        mut req: CorporateActionRequest,
    ) -> Result<Vec<CorporateAction>, TigerError> {
        req.action_type = "split".into();
        self.get_corporate_action(req).await
    }

    /// 公司行动 - 分红。wire: corporate_action (action_type=dividend)
    pub async fn get_corporate_dividend(
        &self,
        mut req: CorporateActionRequest,
    ) -> Result<Vec<CorporateAction>, TigerError> {
        req.action_type = "dividend".into();
        self.get_corporate_action(req).await
    }

    /// 公司行动 - 财报日历。wire: corporate_action (action_type=earning)
    pub async fn get_corporate_earnings_calendar(
        &self,
        mut req: CorporateActionRequest,
    ) -> Result<Vec<CorporateAction>, TigerError> {
        req.action_type = "earning".into();
        self.get_corporate_action(req).await
    }

    // ========== 财务 / 日历 ==========

    /// 获取日级财务数据
    pub async fn get_financial_daily(
        &self,
        req: FinancialDailyRequest,
    ) -> Result<Vec<FinancialDailyItem>, TigerError> {
        self.call_into("financial_daily", req).await
    }

    /// 获取财报数据
    pub async fn get_financial_report(
        &self,
        req: FinancialReportRequest,
    ) -> Result<Vec<FinancialReportItem>, TigerError> {
        self.call_into("financial_report", req).await
    }

    /// 财报币种。wire: financial_currency
    pub async fn get_financial_currency(&self, req: FinancialCurrencyRequest) -> Result<Vec<FinancialCurrency>, TigerError> {
        self.call_into("financial_currency", req).await
    }

    /// 汇率数据。wire: financial_exchange_rate
    pub async fn get_financial_exchange_rate(&self, req: FinancialExchangeRateRequest) -> Result<Vec<ExchangeRate>, TigerError> {
        self.call_into("financial_exchange_rate", req).await
    }

    /// 交易日历。wire: trading_calendar
    pub async fn get_trading_calendar(&self, req: TradingCalendarRequest) -> Result<Vec<TradingCalendarItem>, TigerError> {
        self.call_into("trading_calendar", req).await
    }

    // ========== 资金流向 ==========

    /// 获取资金流向
    pub async fn get_capital_flow(
        &self,
        symbol: &str,
        market: &str,
        period: &str,
    ) -> Result<Option<CapitalFlow>, TigerError> {
        self.call_optional(
            "capital_flow",
            serde_json::json!({ "symbol": symbol, "market": market, "period": period }),
        )
        .await
    }

    /// 获取资金分布
    pub async fn get_capital_distribution(
        &self,
        symbol: &str,
        market: &str,
    ) -> Result<Option<CapitalDistribution>, TigerError> {
        self.call_optional(
            "capital_distribution",
            serde_json::json!({ "symbol": symbol, "market": market }),
        )
        .await
    }

    // ========== 扫描 / 权限 / 其他 ==========

    /// 选股扫描（v1.0）
    pub async fn market_scanner(
        &self,
        req: MarketScannerRequest,
    ) -> Result<Option<ScannerResult>, TigerError> {
        self.call_optional_versioned("market_scanner", req, Some(VERSION_V1)).await
    }

    /// 获取行情权限（老接口）
    pub async fn grab_quote_permission(&self) -> Result<Vec<QuotePermission>, TigerError> {
        self.call_into("grab_quote_permission", serde_json::json!({})).await
    }

    /// 扫描器可用标签。wire: market_scanner_tags
    pub async fn get_market_scanner_tags(&self, req: MarketScannerTagsRequest) -> Result<Option<MarketScannerTags>, TigerError> {
        self.call_optional("market_scanner_tags", req).await
    }

    /// 隔夜行情。wire: quote_overnight
    pub async fn get_quote_overnight(&self, req: QuoteOvernightRequest) -> Result<Vec<QuoteOvernight>, TigerError> {
        self.call_into("quote_overnight", req).await
    }

    // ========== 内部辅助 ==========

    pub async fn call_optional<T, P>(&self, method: &str, params: P) -> Result<Option<T>, TigerError>
    where
        T: serde::de::DeserializeOwned,
        P: Serialize,
    {
        self.call_optional_versioned(method, params, None).await
    }

    pub async fn call_optional_versioned<T, P>(
        &self,
        method: &str,
        params: P,
        version: Option<&str>,
    ) -> Result<Option<T>, TigerError>
    where
        T: serde::de::DeserializeOwned,
        P: Serialize,
    {
        let biz = serde_json::to_string(&params)
            .map_err(|e| TigerError::Config(format!("serialize biz params failed: {}", e)))?;
        let req = match version {
            Some(v) => ApiRequest::with_version(method, biz, v),
            None => ApiRequest::new(method, biz),
        };
        let resp = self.http_client.execute_request(&req).await?;
        match resp.data {
            None => Ok(None),
            Some(v) if v.is_null() => Ok(None),
            Some(v) => {
                let parsed: T = decode_value(v)?;
                Ok(Some(parsed))
            }
        }
    }
}

/// 把 `data` 字段解码为目标类型 T，兼容服务端偶发双重编码（字符串包 JSON）。
/// 当 data 为 None / Null 时返回 T::default()。
fn unmarshal_data<T>(data: Option<Value>) -> Result<T, TigerError>
where
    T: serde::de::DeserializeOwned + Default,
{
    match data {
        None => Ok(T::default()),
        Some(v) if v.is_null() => Ok(T::default()),
        Some(v) => decode_value(v),
    }
}


#[cfg(test)]
mod tests;
