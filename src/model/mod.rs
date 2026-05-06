//! 数据模型模块，包含枚举类型、合约、订单、持仓、行情响应、交易响应等。

pub mod enums;
pub mod contract;
pub mod order;
pub mod position;
pub mod quote;
pub mod trade;

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
    Brief, CapitalDistribution, CapitalFlow, CapitalFlowItem, CorporateAction, CorporateActionRequest,
    Depth, DepthLevel, FinancialDailyItem, FinancialDailyRequest, FinancialReportItem,
    FinancialReportRequest, FutureContractInfo, FutureExchange, FutureKline, FutureKlineItem,
    FutureKlineRequest, FutureQuote, Kline, KlineItem, MarketScannerRequest, MarketState,
    OptionBrief, OptionChain, OptionChainRow, OptionExpiration, OptionKline, OptionLeg as OptionChainLeg,
    QuotePermission, ScannerDataRow, ScannerResult, ScannerResultItem, Timeline, TimelineBucket,
    TimelineItem, TradeTick, TradeTickItem,
};
pub use trade::{
    Asset, AssetSegment, CurrencyAsset, OrderIdResult, PlaceOrderResult, PreviewResult, PrimeAsset,
    PrimeAssetSegment, Transaction,
};
