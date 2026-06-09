//! 数据模型模块，包含枚举类型、合约、订单、持仓、行情响应、交易响应等。

pub mod enums;
pub mod contract;
pub mod order;
pub mod position;
pub mod quote;
pub mod quote_requests;
pub mod trade;
pub mod trade_requests;
pub use quote_requests::*;
pub use trade_requests::*;

// Re-export commonly used types at the model module root.
pub use contract::{
    Contract, TickSize, cash_contract, fund_contract, future_contract, option_contract,
    option_contract_by_symbol, stock_contract, warrant_contract,
};
pub use order::{
    AlgoParams, AlgoParamsRequest, Order, OrderLeg, OrderLegRequest, OrderRequest,
    algo_order, auction_limit_order, auction_market_order, limit_order, market_order,
    new_order_leg, stop_limit_order, stop_order, trail_order,
};
pub use position::Position;
pub use quote::{
    AddonActivePlan, AddonEffectiveEntitlement, AddonEntitlement, AddonInfo,
    Brief, BrokerDetail, CapitalDistribution, CapitalFlow, CapitalFlowItem, CorporateAction,
    CorporateActionRequest, Depth, DepthLevel, ExchangeRate, FinancialCurrency,
    FinancialDailyItem, FinancialDailyRequest, FinancialReportItem, FinancialReportRequest,
    FundContractInfo, FundHistoryQuote, FundQuote,
    FutureContractInfo, FutureDepth, FutureExchange, FutureKline, FutureKlineItem,
    FutureKlineRequest, FutureMainContractHistory, FutureQuote,
    FutureTradingSegment, FutureTradingTime, FutureTradeTickItem,
    IndustryItem, IndustryStock,
    Kline, KlineItem, KlineQuota, KlineQuotaDetail,
    MarketScannerRequest, MarketScannerTag, MarketScannerTags, MarketState,
    OptionAnalysis, OptionBrief, OptionChain, OptionChainRow, OptionExpiration, OptionKline,
    OptionLeg as OptionChainLeg, OptionSymbol, OptionVolatilityPoint,
    QuoteOvernight, QuotePermission,
    ScannerDataRow, ScannerResult, ScannerResultItem,
    ShortInterest, StockBroker, StockBrokerItem, StockDetail, StockFundamental, StockIndustry,
    SymbolName, Timeline, TimelineBucket, TimelineItem, TradeTick, TradeRankItem, TradeTickItem,
    TradingCalendarItem, WarrantBrief, WarrantFilterResult,
};
pub use trade::{
    AggregateAssets, AnalyticsAsset, Asset, AssetSegment, CurrencyAsset, EstimateTradableQuantity,
    ForexOrderResult, FundDetails, FundingHistoryItem, ManagedAccount, OrderIdResult,
    PlaceOrderResult, PositionTransferDetail, PositionTransferExternalRecord,
    PositionTransferRecord, PreviewResult, PrimeAsset, PrimeAssetSegment, SegmentFund,
    SegmentFundAvailableItem, SegmentFundHistoryItem, Transaction, TransferItemResponse,
};
