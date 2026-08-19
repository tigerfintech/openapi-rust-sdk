//! 订单模型定义和构造工具函数。
//!
//! - `Order`：查询类接口返回的订单数据，使用 `#[serde(rename_all = "camelCase")]`。
//! - `OrderRequest`：下单/改单/预览订单接口的请求体，使用 `#[serde(rename_all = "snake_case")]`。

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

// ========== 响应模型（查询类接口返回） ==========

/// 附加订单（止盈/止损）- 响应模型
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderLeg {
    #[serde(default)]
    pub leg_type: String,
    #[serde(default)]
    pub price: f64,
    #[serde(default)]
    pub time_in_force: String,
    #[serde(default)]
    pub quantity: i64,
}

/// 算法订单参数 - 响应模型
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AlgoParams {
    #[serde(default)]
    pub algo_strategy: String,
    #[serde(default)]
    pub start_time: String,
    #[serde(default)]
    pub end_time: String,
    #[serde(default)]
    pub participation_rate: f64,
}

/// 订单响应模型。
///
/// 服务端响应字段为 camelCase。下单/改单请使用 [`OrderRequest`]。
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    #[serde(default)]
    pub account: String,
    /// 全局订单 ID
    #[serde(default)]
    pub id: i64,
    /// 账户自增订单号
    #[serde(default)]
    pub order_id: i64,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub order_type: String,
    #[serde(default)]
    pub total_quantity: i64,
    #[serde(default)]
    pub limit_price: f64,
    #[serde(default)]
    pub aux_price: f64,
    #[serde(default)]
    pub trailing_percent: f64,
    #[serde(default, deserialize_with = "deserialize_order_status")]
    pub status: String,
    #[serde(default)]
    pub filled_quantity: i64,
    #[serde(default)]
    pub avg_fill_price: f64,
    #[serde(default)]
    pub time_in_force: String,
    #[serde(default)]
    pub outside_rth: bool,
    #[serde(default)]
    pub order_legs: Vec<OrderLeg>,
    #[serde(default)]
    pub algo_params: Option<AlgoParams>,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub sec_type: String,
    #[serde(default)]
    pub market: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub expiry: String,
    #[serde(default)]
    pub strike: String,
    #[serde(default)]
    pub right: String,
    #[serde(default)]
    pub identifier: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub commission: f64,
    #[serde(default)]
    pub realized_pnl: f64,
    #[serde(default)]
    pub open_time: i64,
    #[serde(default)]
    pub update_time: i64,
    #[serde(default)]
    pub latest_time: i64,
    #[serde(default)]
    pub remark: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub user_mark: String,
    #[serde(default)]
    pub external_id: String,
    #[serde(default)]
    pub total_quantity_scale: i32,
    #[serde(default)]
    pub filled_quantity_scale: i32,
    #[serde(default)]
    pub filled_cash_amount: f64,
    #[serde(default)]
    pub gst: f64,
    #[serde(default)]
    pub liquidation: bool,
    #[serde(default)]
    pub attr_desc: String,
    #[serde(default)]
    pub attr_list: Vec<String>,
    #[serde(default)]
    pub algo_strategy: String,
    #[serde(default)]
    pub discount: f64,
    #[serde(default)]
    pub replace_status: String,
    #[serde(default)]
    pub cancel_status: String,
    #[serde(default)]
    pub can_modify: bool,
    #[serde(default)]
    pub can_cancel: bool,
    #[serde(default)]
    pub is_open: bool,
    #[serde(default)]
    pub order_discount: f64,
    #[serde(default)]
    pub trading_session_type: String,
    #[serde(default)]
    pub latest_price: f64,
    /// 冰山单：展示数量
    #[serde(default)]
    pub display_size: i64,
    /// 冰山单：最小展示数量
    #[serde(default)]
    pub min_display_size: i64,
    /// 冰山单：价检间隔（秒）
    #[serde(default)]
    pub check_intervals: i64,
    /// 冰山单：价格类型（LIMIT_PRICE / ASK_PRICE / BID_PRICE / LATEST_PRICE）
    #[serde(default)]
    pub price_type: String,
    /// 冰山单：生效开始时间（epoch ms）
    #[serde(default)]
    pub start_time: i64,
    /// 冰山单：生效结束时间（epoch ms）
    #[serde(default)]
    pub end_time: i64,
}

// ========== 自定义反序列化器 ==========

/// 处理服务端偶尔返回整数状态码的情况，将其转换为 Java SDK 枚举字符串名。
///
/// 整数映射表：-2=Invalid, -1=Initial, 3=PendingCancel, 4=Cancelled,
///             5=Submitted, 6=Filled, 7=Inactive, 8=PendingSubmit
fn deserialize_order_status<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let v = Value::deserialize(d)?;
    match v {
        Value::String(s) => Ok(s),
        Value::Number(n) => {
            let code = n.as_i64().unwrap_or(0);
            let name = match code {
                -2 => "Invalid",
                -1 => "Initial",
                3 => "PendingCancel",
                4 => "Cancelled",
                5 => "Submitted",
                6 => "Filled",
                7 => "Inactive",
                8 => "PendingSubmit",
                _ => "Unknown",
            };
            Ok(name.to_string())
        }
        _ => Ok(String::new()),
    }
}

// ========== 请求模型（下单/改单/预览） ==========

/// 附加订单（止盈/止损）- 请求模型
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct OrderLegRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leg_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,
}

/// MLEG 组合单子合约腿（对应 Java ContractLeg）
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ContractLegRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// 合约腿比例（必须大于 0）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ratio: Option<i32>,
}

/// 算法订单参数 - 请求模型 (TWAP / VWAP).
///
/// 字段与 Python SDK 的 `AlgoParams` 对齐。SDK 内部通过自定义 `Serialize`
/// 实现把这个结构体序列化成网关期望的 `[{tag, value}, ...]` 数组形式
/// (与 Python SDK 的 `AlgoParams.to_dict` 保持一致),调用方直接传自然的
/// 结构体即可。
///
/// 注意: `algo_strategy` 不属于 `algo_params` —— 它是 [`OrderRequest`]
/// 顶层字段。老代码里如果误设 `AlgoParams.algo_strategy` 会被静默丢弃,
/// 请改用 `OrderRequest.algo_strategy`。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AlgoParamsRequest {
    /// 开始时间 (epoch-ms, TWAP / VWAP 专用)
    pub start_time: Option<i64>,
    /// 结束时间 (epoch-ms, TWAP / VWAP 专用)
    pub end_time: Option<i64>,
    /// 是否尽可能减少交易次数 (VWAP 专用)
    pub no_take_liq: Option<bool>,
    /// 是否允许生效时间结束后继续完成成交 (TWAP / VWAP 专用)
    pub allow_past_end_time: Option<bool>,
    /// 参与率 (VWAP 专用, 0.01–0.5)
    pub participation_rate: Option<f64>,
}

impl Serialize for AlgoParamsRequest {
    /// 序列化成 `[{tag, value}, ...]` 数组 —— 网关期望的形状。
    ///
    /// 直接发 object 会触发 `biz param error(failed to parse parameters in
    /// biz_content)`。Python SDK 的 `AlgoParams.to_dict` 用同样的形状。
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        // 未设置的字段(None)直接跳过,与 Python SDK 保持一致的语义。
        let mut entries: Vec<(&'static str, serde_json::Value)> = Vec::with_capacity(5);
        if let Some(v) = self.start_time {
            entries.push(("start_time", serde_json::Value::from(v)));
        }
        if let Some(v) = self.end_time {
            entries.push(("end_time", serde_json::Value::from(v)));
        }
        if let Some(v) = self.no_take_liq {
            entries.push(("no_take_liq", serde_json::Value::from(v)));
        }
        if let Some(v) = self.allow_past_end_time {
            entries.push(("allow_past_end_time", serde_json::Value::from(v)));
        }
        if let Some(v) = self.participation_rate {
            entries.push(("participation_rate", serde_json::Value::from(v)));
        }

        let mut seq = serializer.serialize_seq(Some(entries.len()))?;
        for (tag, value) in entries {
            #[derive(Serialize)]
            struct TagValue<'a> {
                tag: &'a str,
                value: serde_json::Value,
            }
            seq.serialize_element(&TagValue { tag, value })?;
        }
        seq.end()
    }
}

/// 订单请求模型。
///
/// 服务端请求体字段为 snake_case。此结构体用于 `place_order` / `preview_order`
/// / `modify_order` 接口。查询返回请使用 [`Order`]。
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct OrderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// 全局订单 ID（修改订单时必填）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_quantity: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aux_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outside_rth: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_legs: Option<Vec<OrderLegRequest>>,
    /// 算法策略 (TWAP / VWAP) —— 顶层字段,不在 algo_params 里
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algo_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algo_params: Option<AlgoParamsRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_mark: Option<String>,
    /// 冰山单：展示数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_size: Option<i64>,
    /// 冰山单：最小展示数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_display_size: Option<i64>,
    /// 冰山单：价检间隔（秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_intervals: Option<i64>,
    /// 冰山单：价格类型（LIMIT_PRICE / ASK_PRICE / BID_PRICE / LATEST_PRICE）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_type: Option<String>,
    /// 冰山单：生效开始时间（epoch ms）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    /// 冰山单：生效结束时间（epoch ms）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    /// 机构账户交易密钥（client 层自动填充默认值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    /// 限价偏移（adjust_limit），用于 STP 等场景
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjust_limit: Option<f64>,
    /// 订单到期时间（epoch ms），GTD 单必填
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_time: Option<i64>,
    /// 交易时段类型（如 RTH / ETH / ALL）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trading_session_type: Option<String>,
    /// 交易所
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange: Option<String>,
    /// 合约乘数（期权/期货）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiplier: Option<String>,
    /// 本地合约代码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_symbol: Option<String>,
    /// 机构分配账户列表（wire: alloc_accounts，JSON 数组）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alloc_accounts: Option<Vec<String>>,
    /// 机构分配份额列表（与 alloc_accounts 一一对应，wire: alloc_shares，JSON 数组）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alloc_shares: Option<Vec<f64>>,
    /// 数量精度（小数点后位数）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_quantity_scale: Option<i32>,
    /// 附属订单类型（PROFIT_TAKER / STOP_LOSS / BRACKET 等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attach_type: Option<String>,
    /// 止盈订单 ID
    #[serde(
        rename = "profit_taker_orderId",
        skip_serializing_if = "Option::is_none"
    )]
    pub profit_taker_order_id: Option<i64>,
    /// 止盈价格
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_taker_price: Option<f64>,
    /// 止盈 TIF
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_taker_tif: Option<String>,
    /// 止盈是否支持盘外交易
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_taker_rth: Option<bool>,
    /// 止损订单类型（STP / STP LMT / TRAIL 等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss_order_type: Option<String>,
    /// 止损订单 ID
    #[serde(rename = "stop_loss_orderId", skip_serializing_if = "Option::is_none")]
    pub stop_loss_order_id: Option<i64>,
    /// 止损触发价
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss_price: Option<f64>,
    /// 止损限价（STP LMT 用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss_limit_price: Option<f64>,
    /// 止损 TIF
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss_tif: Option<String>,
    /// 止损追踪百分比（TRAIL 用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss_trailing_percent: Option<f64>,
    /// 止损追踪金额（TRAIL 用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss_trailing_amount: Option<f64>,
    /// 组合单类型（如 MLEG）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combo_type: Option<String>,
    /// MLEG 多腿组合的子合约列表（wire: contract_legs，对应 Java ContractLeg）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_legs: Option<Vec<ContractLegRequest>>,
    /// OCA 组关联订单列表（wire: oca_orders，对应 Java List<TradeOrderModel>）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oca_orders: Option<Vec<Box<OrderRequest>>>,
    /// 现金金额（按金额下单用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cash_amount: Option<f64>,
}

// ========== 订单请求构造工具函数 ==========

/// 构造市价单
pub fn market_order(
    account: &str,
    symbol: &str,
    sec_type: &str,
    action: &str,
    quantity: i64,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("MKT".to_string()),
        total_quantity: Some(quantity),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造限价单
pub fn limit_order(
    account: &str,
    symbol: &str,
    sec_type: &str,
    action: &str,
    quantity: i64,
    limit_price: f64,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("LMT".to_string()),
        total_quantity: Some(quantity),
        limit_price: Some(limit_price),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造止损单
pub fn stop_order(
    account: &str,
    symbol: &str,
    sec_type: &str,
    action: &str,
    quantity: i64,
    aux_price: f64,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("STP".to_string()),
        total_quantity: Some(quantity),
        aux_price: Some(aux_price),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造止损限价单
pub fn stop_limit_order(
    account: &str,
    symbol: &str,
    sec_type: &str,
    action: &str,
    quantity: i64,
    limit_price: f64,
    aux_price: f64,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("STP_LMT".to_string()),
        total_quantity: Some(quantity),
        limit_price: Some(limit_price),
        aux_price: Some(aux_price),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造跟踪止损单
pub fn trail_order(
    account: &str,
    symbol: &str,
    sec_type: &str,
    action: &str,
    quantity: i64,
    trailing_percent: f64,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("TRAIL".to_string()),
        total_quantity: Some(quantity),
        trailing_percent: Some(trailing_percent),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造竞价限价单
pub fn auction_limit_order(
    account: &str,
    symbol: &str,
    sec_type: &str,
    action: &str,
    quantity: i64,
    limit_price: f64,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("AL".to_string()),
        total_quantity: Some(quantity),
        limit_price: Some(limit_price),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造竞价市价单
pub fn auction_market_order(
    account: &str,
    symbol: &str,
    sec_type: &str,
    action: &str,
    quantity: i64,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("AM".to_string()),
        total_quantity: Some(quantity),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造算法订单 (TWAP / VWAP)。
///
/// `algo_type` 会同时写入 [`OrderRequest::order_type`] 和
/// [`OrderRequest::algo_strategy`] —— 后者是网关期望的顶层策略字段,
/// 不在 `algo_params` 里。
pub fn algo_order(
    account: &str,
    symbol: &str,
    sec_type: &str,
    action: &str,
    quantity: i64,
    limit_price: f64,
    algo_type: &str,
    params: AlgoParamsRequest,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some(algo_type.to_string()),
        algo_strategy: Some(algo_type.to_string()),
        total_quantity: Some(quantity),
        limit_price: Some(limit_price),
        algo_params: Some(params),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造冰山单（最简参数）
pub fn iceberg_order(
    account: &str,
    symbol: &str,
    sec_type: &str,
    action: &str,
    quantity: i64,
    limit_price: f64,
    display_size: i64,
    min_display_size: Option<i64>,
    check_intervals: Option<i64>,
    price_type: Option<&str>,
    start_time: Option<i64>,
    end_time: Option<i64>,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("ICEBERG".to_string()),
        total_quantity: Some(quantity),
        limit_price: Some(limit_price),
        time_in_force: Some("DAY".to_string()),
        display_size: Some(display_size),
        price_type: Some(price_type.unwrap_or("LIMIT_PRICE").to_string()),
        min_display_size,
        check_intervals,
        start_time: start_time.filter(|&v| v > 0),
        end_time: end_time.filter(|&v| v > 0),
        ..OrderRequest::default()
    }
}

/// 构造按金额市价单
pub fn market_order_by_amount(
    account: &str,
    symbol: &str,
    sec_type: &str,
    action: &str,
    amount: f64,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("MKT".to_string()),
        total_quantity: Some(0),
        cash_amount: Some(amount),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造按金额限价单
pub fn limit_order_by_amount(
    account: &str,
    symbol: &str,
    sec_type: &str,
    action: &str,
    amount: f64,
    limit_price: f64,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("LMT".to_string()),
        total_quantity: Some(0),
        cash_amount: Some(amount),
        limit_price: Some(limit_price),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造按价差跟踪止损单（使用 aux_price 而非百分比）
pub fn trail_order_by_price(
    account: &str,
    symbol: &str,
    sec_type: &str,
    action: &str,
    quantity: i64,
    aux_price: f64,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("TRAIL".to_string()),
        total_quantity: Some(quantity),
        aux_price: Some(aux_price),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造限价单 + 附加止盈/止损腿（bracket 单，最多 2 腿）
pub fn limit_order_with_legs(
    account: &str,
    symbol: &str,
    sec_type: &str,
    action: &str,
    quantity: i64,
    limit_price: f64,
    order_legs: Vec<OrderLegRequest>,
) -> OrderRequest {
    assert!(order_legs.len() <= 2, "At most 2 order legs are supported");
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("LMT".to_string()),
        total_quantity: Some(quantity),
        limit_price: Some(limit_price),
        order_legs: Some(order_legs),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造多腿组合单（MLEG）
pub fn combo_order(
    account: &str,
    action: &str,
    quantity: i64,
    order_type: &str,
    contract_legs: Vec<ContractLegRequest>,
    combo_type: Option<&str>,
    limit_price: Option<f64>,
    aux_price: Option<f64>,
    trailing_percent: Option<f64>,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        sec_type: Some("MLEG".to_string()),
        action: Some(action.to_string()),
        order_type: Some(order_type.to_string()),
        total_quantity: Some(quantity),
        limit_price,
        aux_price,
        trailing_percent,
        contract_legs: Some(contract_legs),
        combo_type: combo_type.map(|s| s.to_string()),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造 OCA（One-Cancels-All）单
pub fn oca_order(
    account: &str,
    symbol: &str,
    sec_type: &str,
    action: &str,
    quantity: i64,
    oca_orders: Vec<Box<OrderRequest>>,
) -> OrderRequest {
    OrderRequest {
        account: Some(account.to_string()),
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        order_type: Some("OCA".to_string()),
        total_quantity: Some(quantity),
        oca_orders: Some(oca_orders),
        time_in_force: Some("DAY".to_string()),
        ..OrderRequest::default()
    }
}

/// 构造多腿组合单的子腿
pub fn contract_leg(
    symbol: &str,
    sec_type: &str,
    action: &str,
    ratio: i32,
    expiry: Option<&str>,
    strike: Option<&str>,
    right: Option<&str>,
) -> ContractLegRequest {
    ContractLegRequest {
        symbol: Some(symbol.to_string()),
        sec_type: Some(sec_type.to_string()),
        action: Some(action.to_string()),
        ratio: Some(ratio),
        expiry: expiry.map(|s| s.to_string()),
        strike: strike.map(|s| s.to_string()),
        right: right.map(|s| s.to_string()),
    }
}

/// 构造附加订单（止盈/止损）
pub fn new_order_leg(leg_type: &str, price: f64, time_in_force: &str) -> OrderLegRequest {
    OrderLegRequest {
        leg_type: Some(leg_type.to_string()),
        price: Some(price),
        time_in_force: Some(time_in_force.to_string()),
        quantity: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_status_integer_deserialization() {
        // 服务端偶尔返回整数状态码
        let json = r#"{"status": 6}"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.status, "Filled");

        let json = r#"{"status": 5}"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.status, "Submitted");

        let json = r#"{"status": 8}"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.status, "PendingSubmit");

        let json = r#"{"status": -2}"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.status, "Invalid");

        // 字符串形式仍然正常工作
        let json = r#"{"status": "Filled"}"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.status, "Filled");
    }

    #[test]
    fn test_order_request_serializes_to_snake_case() {
        let order = limit_order("ACC123", "AAPL", "STK", "BUY", 100, 150.50);
        let json_value: serde_json::Value = serde_json::to_value(&order).unwrap();
        let obj = json_value.as_object().unwrap();

        // snake_case 请求体
        assert!(obj.contains_key("sec_type"), "request should use sec_type");
        assert!(
            obj.contains_key("order_type"),
            "request should use order_type"
        );
        assert!(
            obj.contains_key("total_quantity"),
            "request should use total_quantity"
        );
        assert!(
            obj.contains_key("limit_price"),
            "request should use limit_price"
        );
        assert!(
            obj.contains_key("time_in_force"),
            "request should use time_in_force"
        );

        // 不应出现 camelCase
        assert!(!obj.contains_key("secType"));
        assert!(!obj.contains_key("orderType"));
        assert!(!obj.contains_key("totalQuantity"));
    }

    #[test]
    fn test_order_response_parses_camel_case() {
        let json = r#"{
            "account": "ACC123",
            "id": 42519413060422656,
            "orderId": 143,
            "action": "BUY",
            "orderType": "MKT",
            "totalQuantity": 100,
            "filledQuantity": 100,
            "avgFillPrice": 543.5,
            "timeInForce": "DAY",
            "outsideRth": false,
            "symbol": "00700",
            "secType": "STK",
            "market": "HK",
            "status": "Filled",
            "commission": 92.38,
            "realizedPnl": 0.0,
            "name": "TENCENT",
            "identifier": "00700",
            "source": "openapi",
            "userMark": "test001",
            "openTime": 1773296577000,
            "updateTime": 1773590598000
        }"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.account, "ACC123");
        assert_eq!(order.id, 42519413060422656);
        assert_eq!(order.order_id, 143);
        assert_eq!(order.total_quantity, 100);
        assert_eq!(order.filled_quantity, 100);
        assert_eq!(order.avg_fill_price, 543.5);
        assert_eq!(order.status, "Filled");
        assert_eq!(order.sec_type, "STK");
        assert_eq!(order.name, "TENCENT");
        assert_eq!(order.user_mark, "test001");
        assert_eq!(order.open_time, 1773296577000);
    }

    #[test]
    fn test_market_order_helper() {
        let o = market_order("ACC", "AAPL", "STK", "BUY", 100);
        assert_eq!(o.total_quantity, Some(100));
        assert_eq!(o.order_type, Some("MKT".to_string()));
    }

    #[test]
    fn test_limit_order_helper() {
        let o = limit_order("ACC", "AAPL", "STK", "SELL", 50, 155.0);
        assert_eq!(o.order_type, Some("LMT".to_string()));
        assert_eq!(o.limit_price, Some(155.0));
    }

    #[test]
    fn test_stop_order_helper() {
        let o = stop_order("ACC", "AAPL", "STK", "SELL", 100, 140.0);
        assert_eq!(o.order_type, Some("STP".to_string()));
        assert_eq!(o.aux_price, Some(140.0));
    }

    #[test]
    fn test_new_order_leg_helper() {
        let leg = new_order_leg("PROFIT", 160.0, "GTC");
        assert_eq!(leg.leg_type, Some("PROFIT".to_string()));
        assert_eq!(leg.price, Some(160.0));
    }

    #[test]
    fn test_iceberg_order_basic() {
        let o = iceberg_order(
            "ACC", "AAPL", "STK", "BUY", 1000, 180.0, 100, None, None, None, None, None,
        );
        assert_eq!(o.order_type, Some("ICEBERG".to_string()));
        assert_eq!(o.total_quantity, Some(1000));
        assert_eq!(o.limit_price, Some(180.0));
        assert_eq!(o.display_size, Some(100));
        assert_eq!(o.time_in_force, Some("DAY".to_string()));
        assert_eq!(o.min_display_size, None);
        assert_eq!(o.start_time, None);
    }

    #[test]
    fn test_iceberg_order_with_optional_fields() {
        let start_time: i64 = 1782293585902;
        let end_time: i64 = 1782297185902;
        let o = iceberg_order(
            "ACC",
            "AAPL",
            "STK",
            "BUY",
            1000,
            180.0,
            100,
            Some(50),
            Some(30),
            Some("LIMIT_PRICE"),
            Some(start_time),
            Some(end_time),
        );
        assert_eq!(o.order_type, Some("ICEBERG".to_string()));
        assert_eq!(o.display_size, Some(100));
        assert_eq!(o.min_display_size, Some(50));
        assert_eq!(o.check_intervals, Some(30));
        assert_eq!(o.price_type, Some("LIMIT_PRICE".to_string()));
        assert_eq!(o.start_time, Some(start_time));
        assert_eq!(o.end_time, Some(end_time));
    }

    #[test]
    fn test_iceberg_order_no_time_window() {
        let o = iceberg_order(
            "ACC",
            "AAPL",
            "STK",
            "BUY",
            1000,
            180.0,
            100,
            Some(50),
            Some(30),
            Some("ASK_PRICE"),
            None,
            None,
        );
        assert_eq!(o.price_type, Some("ASK_PRICE".to_string()));
        assert_eq!(o.start_time, None);
        assert_eq!(o.end_time, None);
    }

    #[test]
    fn test_iceberg_request_serializes_snake_case() {
        let o = iceberg_order(
            "ACC", "AAPL", "STK", "BUY", 1000, 180.0, 100, None, None, None, None, None,
        );
        let json_value: serde_json::Value = serde_json::to_value(&o).unwrap();
        let obj = json_value.as_object().unwrap();
        assert!(
            obj.contains_key("display_size"),
            "should serialize display_size"
        );
        assert!(
            !obj.contains_key("min_display_size"),
            "None fields should be omitted"
        );
    }

    #[test]
    fn test_iceberg_response_deserializes() {
        let json = r#"{
            "orderType": "ICEBERG",
            "displaySize": 100,
            "minDisplaySize": 50,
            "checkIntervals": 30,
            "priceType": "LIMIT_PRICE",
            "startTime": 1782293585902,
            "endTime": 1782297185902
        }"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.order_type, "ICEBERG");
        assert_eq!(order.display_size, 100);
        assert_eq!(order.min_display_size, 50);
        assert_eq!(order.check_intervals, 30);
        assert_eq!(order.price_type, "LIMIT_PRICE");
        assert_eq!(order.start_time, 1782293585902);
        assert_eq!(order.end_time, 1782297185902);
    }

    #[test]
    fn test_order_request_skip_none_fields() {
        let order = market_order("ACC", "AAPL", "STK", "BUY", 100);
        let json_value: serde_json::Value = serde_json::to_value(&order).unwrap();
        let obj = json_value.as_object().unwrap();

        // 必填字段存在
        assert!(obj.contains_key("symbol"));
        assert!(obj.contains_key("sec_type"));
        assert!(obj.contains_key("order_type"));
        assert!(obj.contains_key("total_quantity"));

        // None 字段不应出现
        assert!(!obj.contains_key("id"));
        assert!(!obj.contains_key("limit_price"));
        assert!(!obj.contains_key("aux_price"));
    }

    // ========== 未覆盖的构造函数测试 ==========

    #[test]
    fn test_stop_limit_order_helper() {
        let o = stop_limit_order("ACC", "AAPL", "STK", "SELL", 100, 145.0, 140.0);
        assert_eq!(o.order_type, Some("STP_LMT".to_string()));
        assert_eq!(o.limit_price, Some(145.0));
        assert_eq!(o.aux_price, Some(140.0));
        assert_eq!(o.total_quantity, Some(100));
        assert_eq!(o.time_in_force, Some("DAY".to_string()));
    }

    #[test]
    fn test_trail_order_helper() {
        let o = trail_order("ACC", "AAPL", "STK", "SELL", 100, 5.0);
        assert_eq!(o.order_type, Some("TRAIL".to_string()));
        assert_eq!(o.trailing_percent, Some(5.0));
        assert_eq!(o.total_quantity, Some(100));
    }

    #[test]
    fn test_auction_limit_order_helper() {
        let o = auction_limit_order("ACC", "AAPL", "STK", "BUY", 100, 150.0);
        assert_eq!(o.order_type, Some("AL".to_string()));
        assert_eq!(o.limit_price, Some(150.0));
    }

    #[test]
    fn test_auction_market_order_helper() {
        let o = auction_market_order("ACC", "AAPL", "STK", "SELL", 50);
        assert_eq!(o.order_type, Some("AM".to_string()));
        assert_eq!(o.total_quantity, Some(50));
    }

    #[test]
    fn test_algo_order_helper() {
        // algo_strategy 现在是 OrderRequest 顶层字段(不在 algo_params 里);
        // start_time / end_time 用 epoch-ms(i64) 而不是字符串。
        let params = AlgoParamsRequest {
            start_time: Some(1_700_000_000_000),
            end_time: Some(1_700_003_600_000),
            participation_rate: Some(10.0),
            ..Default::default()
        };
        let o = algo_order("ACC", "AAPL", "STK", "BUY", 100, 150.0, "TWAP", params);
        assert_eq!(o.order_type, Some("TWAP".to_string()));
        assert_eq!(o.algo_strategy, Some("TWAP".to_string()));
        assert_eq!(o.limit_price, Some(150.0));
        assert!(o.algo_params.is_some());
        assert_eq!(
            o.algo_params.as_ref().unwrap().participation_rate,
            Some(10.0)
        );
    }

    /// AlgoParamsRequest 必须序列化成 `[{tag, value}, ...]` 数组
    /// (网关期望的形状,与 Python SDK 的 AlgoParams.to_dict 一致)。
    #[test]
    fn test_algo_params_serializes_as_tag_value_array() {
        let p = AlgoParamsRequest {
            start_time: Some(1_700_000_000_000),
            end_time: Some(1_700_003_600_000),
            participation_rate: Some(0.1),
            allow_past_end_time: Some(true),
            ..Default::default()
        };
        let json = serde_json::to_value(&p).unwrap();
        let arr = json.as_array().expect("expected array, got object");
        // Should be 4 entries — the 4 fields set above; None fields skipped.
        assert_eq!(arr.len(), 4, "unexpected count: {:?}", arr);
        // Collect tag→value map for assertions.
        let mut tags = std::collections::HashMap::new();
        for entry in arr {
            let tag = entry["tag"].as_str().unwrap().to_string();
            tags.insert(tag, entry["value"].clone());
        }
        assert_eq!(tags["start_time"], serde_json::json!(1_700_000_000_000i64));
        assert_eq!(tags["end_time"], serde_json::json!(1_700_003_600_000i64));
        assert_eq!(tags["participation_rate"], serde_json::json!(0.1));
        assert_eq!(tags["allow_past_end_time"], serde_json::json!(true));
    }

    /// 未设置的字段(None)应被跳过,不出现在数组里。
    #[test]
    fn test_algo_params_omits_none_fields() {
        let p = AlgoParamsRequest {
            start_time: Some(100),
            ..Default::default()
        };
        let json = serde_json::to_value(&p).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["tag"].as_str().unwrap(), "start_time");
    }

    /// 嵌套在 OrderRequest 里,algo_params 也应序列化成数组;
    /// algo_strategy 应在顶层。
    #[test]
    fn test_order_request_nests_algo_params_as_array() {
        let order = OrderRequest {
            symbol: Some("AAPL".to_string()),
            sec_type: Some("STK".to_string()),
            action: Some("BUY".to_string()),
            order_type: Some("TWAP".to_string()),
            total_quantity: Some(100),
            algo_strategy: Some("TWAP".to_string()),
            algo_params: Some(AlgoParamsRequest {
                start_time: Some(1_700_000_000_000),
                end_time: Some(1_700_003_600_000),
                ..Default::default()
            }),
            ..Default::default()
        };
        let json = serde_json::to_value(&order).unwrap();
        assert!(
            json["algo_params"].is_array(),
            "algo_params should be array, got {:?}",
            json["algo_params"]
        );
        assert_eq!(json["algo_strategy"], serde_json::json!("TWAP"));
    }

    #[test]
    fn test_market_order_by_amount_helper() {
        let o = market_order_by_amount("ACC", "AAPL", "STK", "BUY", 10000.0);
        assert_eq!(o.order_type, Some("MKT".to_string()));
        assert_eq!(o.total_quantity, Some(0));
        assert_eq!(o.cash_amount, Some(10000.0));
    }

    #[test]
    fn test_limit_order_by_amount_helper() {
        let o = limit_order_by_amount("ACC", "AAPL", "STK", "BUY", 10000.0, 150.0);
        assert_eq!(o.order_type, Some("LMT".to_string()));
        assert_eq!(o.cash_amount, Some(10000.0));
        assert_eq!(o.limit_price, Some(150.0));
    }

    #[test]
    fn test_trail_order_by_price_helper() {
        let o = trail_order_by_price("ACC", "AAPL", "STK", "SELL", 100, 5.0);
        assert_eq!(o.order_type, Some("TRAIL".to_string()));
        assert_eq!(o.aux_price, Some(5.0));
        assert_eq!(o.trailing_percent, None);
    }

    #[test]
    fn test_limit_order_with_legs_helper() {
        let legs = vec![
            new_order_leg("PROFIT", 160.0, "GTC"),
            new_order_leg("LOSS", 140.0, "GTC"),
        ];
        let o = limit_order_with_legs("ACC", "AAPL", "STK", "BUY", 100, 150.0, legs);
        assert_eq!(o.order_type, Some("LMT".to_string()));
        assert_eq!(o.limit_price, Some(150.0));
        assert!(o.order_legs.is_some());
        assert_eq!(o.order_legs.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_combo_order_helper() {
        let legs = vec![contract_leg("AAPL", "STK", "BUY", 1, None, None, None)];
        let o = combo_order(
            "ACC",
            "BUY",
            100,
            "LMT",
            legs,
            Some("GUARDED"),
            Some(150.0),
            None,
            None,
        );
        assert_eq!(o.sec_type, Some("MLEG".to_string()));
        assert_eq!(o.order_type, Some("LMT".to_string()));
        assert_eq!(o.combo_type, Some("GUARDED".to_string()));
        assert_eq!(o.limit_price, Some(150.0));
        assert!(o.contract_legs.is_some());
    }

    #[test]
    fn test_oca_order_helper() {
        let inner = Box::new(market_order("ACC", "AAPL", "STK", "BUY", 100));
        let o = oca_order("ACC", "AAPL", "STK", "BUY", 100, vec![inner]);
        assert_eq!(o.order_type, Some("OCA".to_string()));
        assert!(o.oca_orders.is_some());
        assert_eq!(o.oca_orders.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_contract_leg_helper() {
        let leg = contract_leg(
            "AAPL",
            "OPT",
            "BUY",
            1,
            Some("2024-01-19"),
            Some("150.0"),
            Some("CALL"),
        );
        assert_eq!(leg.symbol, Some("AAPL".to_string()));
        assert_eq!(leg.sec_type, Some("OPT".to_string()));
        assert_eq!(leg.ratio, Some(1));
        assert_eq!(leg.expiry, Some("2024-01-19".to_string()));
        assert_eq!(leg.strike, Some("150.0".to_string()));
        assert_eq!(leg.right, Some("CALL".to_string()));
    }

    #[test]
    fn test_contract_leg_helper_no_optionals() {
        let leg = contract_leg("AAPL", "STK", "BUY", 1, None, None, None);
        assert_eq!(leg.expiry, None);
        assert_eq!(leg.strike, None);
        assert_eq!(leg.right, None);
    }

    #[test]
    fn test_iceberg_order_zero_start_end_time_filtered() {
        // start_time=0 and end_time=0 should be filtered out (filter |&v| v > 0)
        let o = iceberg_order(
            "ACC",
            "AAPL",
            "STK",
            "BUY",
            1000,
            180.0,
            100,
            None,
            None,
            None,
            Some(0),
            Some(0),
        );
        assert_eq!(o.start_time, None);
        assert_eq!(o.end_time, None);
    }

    #[test]
    fn test_iceberg_order_default_price_type() {
        // price_type=None → default "LIMIT_PRICE"
        let o = iceberg_order(
            "ACC", "AAPL", "STK", "BUY", 1000, 180.0, 100, None, None, None, None, None,
        );
        assert_eq!(o.price_type, Some("LIMIT_PRICE".to_string()));
    }
}
