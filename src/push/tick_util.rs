use super::pb::{self, trade_tick_data};

fn part_code_name(code: &str) -> &'static str {
    match code {
        "a" => "NYSE American, LLC (NYSE American)",
        "b" => "NASDAQ OMX BX, Inc. (NASDAQ OMX BX)",
        "c" => "NYSE National, Inc. (NYSE National)",
        "d" => "FINRA Alternative Display Facility (ADF)",
        "h" => "MIAX Pearl Exchange, LLC (MIAX)",
        "i" => "International Securities Exchange, LLC (ISE)",
        "j" => "Cboe EDGA Exchange, Inc. (Cboe EDGA)",
        "k" => "Cboe EDGX Exchange, Inc. (Cboe EDGX)",
        "l" => "Long-Term Stock Exchange, Inc. (LTSE)",
        "m" => "NYSE Chicago, Inc. (NYSE Chicago)",
        "n" => "New York Stock Exchange, LLC (NYSE)",
        "p" => "NYSE Arca, Inc. (NYSE Arca)",
        "s" => "Consolidated Tape System (CTS)",
        "t" => "NASDAQ Stock Market, LLC (NASDAQ)",
        "u" => "Members Exchange, LLC (MEMX)",
        "v" => "Investors' Exchange, LLC. (IEX)",
        "w" => "CBOE Stock Exchange, Inc. (CBSX)",
        "x" => "NASDAQ OMX PSX, Inc. (NASDAQ OMX PSX)",
        "y" => "Cboe BYX Exchange, Inc. (Cboe BYX)",
        "z" => "Cboe BZX Exchange, Inc. (Cboe BZX)",
        _ => "",
    }
}

fn part_code_short_name(code: &str) -> &'static str {
    match code {
        "a" => "AMEX",
        "b" => "BX",
        "c" => "NSX",
        "d" => "ADF",
        "h" => "MIAX",
        "i" => "ISE",
        "j" => "EDGA",
        "k" => "EDGX",
        "l" => "LTSE",
        "m" => "CHO",
        "n" => "NYSE",
        "p" => "ARCA",
        "s" => "CTS",
        "t" => "NSDQ",
        "u" => "MEMX",
        "v" => "IEX",
        "w" => "CBSX",
        "x" => "PSX",
        "y" => "BYX",
        "z" => "BZX",
        _ => "",
    }
}

fn us_trade_cond(ch: char) -> &'static str {
    match ch {
        ' ' => "US_REGULAR_SALE",
        'B' => "US_BUNCHED_TRADE",
        'C' => "US_CASH_TRADE",
        'F' => "US_INTERMARKET_SWEEP",
        'G' => "US_BUNCHED_SOLD_TRADE",
        'H' => "US_PRICE_VARIATION_TRADE",
        'I' => "US_ODD_LOT_TRADE",
        'K' => "US_RULE_127_OR_155_TRADE",
        'L' => "US_SOLD_LAST",
        'M' => "US_MARKET_CENTER_CLOSE_PRICE",
        'N' => "US_NEXT_DAY_TRADE",
        'O' => "US_MARKET_CENTER_OPENING_TRADE",
        'P' => "US_PRIOR_REFERENCE_PRICE",
        'Q' => "US_MARKET_CENTER_OPEN_PRICE",
        'R' => "US_SELLER",
        'T' => "US_FORM_T",
        'U' => "US_EXTENDED_TRADING_HOURS",
        'V' => "US_CONTINGENT_TRADE",
        'W' => "US_AVERAGE_PRICE_TRADE",
        'X' => "US_CROSS_TRADE",
        'Z' => "US_SOLD_OUT_OF_SEQUENCE",
        '0' => "US_ODD_LOST_CROSS_TRADE",
        '4' => "US_DERIVATIVELY_PRICED",
        '5' => "US_MARKET_CENTER_RE_OPENING_TRADE",
        '6' => "US_MARKET_CENTER_CLOSING_TRADE",
        '7' => "US_QUALIFIED_CONTINGENT_TRADE",
        '9' => "US_CONSOLIDATED_LAST_PRICE_PER_LISTING_PACKET",
        _ => "",
    }
}

fn hk_trade_cond(ch: char) -> &'static str {
    match ch {
        ' ' => "HK_AUTOMATCH_NORMAL",
        'D' => "HK_ODD_LOT_TRADE",
        'U' => "HK_AUCTION_TRADE",
        '*' => "HK_OVERSEAS_TRADE",
        'P' => "HK_LATE_TRADE_OFF_EXCHG",
        'M' => "HK_NON_DIRECT_OFF_EXCHG_TRADE",
        'X' => "HK_DIRECT_OFF_EXCHG_TRADE",
        'Y' => "HK_AUTOMATIC_INTERNALIZED",
        _ => "",
    }
}

fn get_trade_cond(is_us: bool, ch: char) -> String {
    let s = if is_us {
        us_trade_cond(ch)
    } else {
        hk_trade_cond(ch)
    };
    if s.is_empty() {
        ch.to_string()
    } else {
        s.to_string()
    }
}

fn is_us_symbol(symbol: &str) -> bool {
    symbol
        .bytes()
        .next()
        .map_or(false, |b| b.is_ascii_uppercase())
        && symbol
            .split_once('.')
            .map_or(symbol, |(prefix, _)| prefix)
            .bytes()
            .all(|b| b.is_ascii_uppercase())
}

/// Resolves a raw cond character to a readable string.
/// Set is_us=true for US stocks, false for HK stocks.
pub fn get_trade_cond_by_code(is_us: bool, ch: char) -> String {
    get_trade_cond(is_us, ch)
}

/// Reports whether symbol is a US stock (no dot separator).
pub fn is_us_stock_symbol(symbol: &str) -> bool {
    is_us_symbol(symbol)
}

/// Returns the full exchange name for a partCode letter, or the raw code if unknown.
pub fn get_part_name_by_code(code: &str) -> String {
    let s = part_code_name(code);
    if s.is_empty() {
        code.to_string()
    } else {
        s.to_string()
    }
}

/// Returns the short exchange name for a partCode letter, or the raw code if unknown.
pub fn get_part_short_name_by_code(code: &str) -> String {
    let s = part_code_short_name(code);
    if s.is_empty() {
        code.to_string()
    } else {
        s.to_string()
    }
}

/// Single decoded tick from a push message.
#[derive(Debug, Clone)]
pub struct PushTick {
    pub sn: i64,
    pub time: i64,
    pub price: f64,
    pub volume: i64,
    pub tick_type: String,
    pub cond: String,
    pub part_code: String,
    pub part_name: String,
}

/// Decoded push trade tick message — equivalent to Java TradeTick.
#[derive(Debug, Clone)]
pub struct PushTradeTick {
    pub symbol: String,
    pub sec_type: String,
    pub quote_level: String,
    pub timestamp: u64,
    pub ticks: Vec<PushTick>,
}

/// Decodes a TradeTickData pb message into a PushTradeTick with resolved partCode/partName.
pub fn convert_trade_tick(src: pb::TradeTickData) -> PushTradeTick {
    if src.sec_type == "FUT" {
        convert_future_tick(src)
    } else {
        convert_stock_tick(src)
    }
}

fn convert_stock_tick(src: pb::TradeTickData) -> PushTradeTick {
    let is_us = is_us_symbol(&src.symbol);
    let denominator = 10f64.powi(src.price_offset);
    let cond_chars: Vec<char> = src.cond.chars().collect();
    let type_chars: Vec<char> = src.r#type.chars().collect();
    let mut current_time: i64 = 0;
    let mut ticks = Vec::with_capacity(src.time.len());

    for i in 0..src.time.len() {
        current_time += src.time[i];
        let raw_code = src.part_code.get(i).map(|s| s.as_str()).unwrap_or("");
        let cond_ch = cond_chars.get(i).copied().unwrap_or(' ');
        ticks.push(PushTick {
            sn: src.sn + i as i64,
            time: current_time,
            price: (src.price_base + src.price[i]) as f64 / denominator,
            volume: src.volume[i],
            tick_type: type_chars.get(i).map(|c| c.to_string()).unwrap_or_default(),
            cond: get_trade_cond(is_us, cond_ch),
            part_code: get_part_short_name_by_code(raw_code),
            part_name: get_part_name_by_code(raw_code),
        });
    }

    PushTradeTick {
        symbol: src.symbol,
        sec_type: src.sec_type,
        quote_level: src.quote_level,
        timestamp: src.timestamp,
        ticks,
    }
}

fn convert_future_tick(src: pb::TradeTickData) -> PushTradeTick {
    let denominator = 10f64.powi(src.price_offset);
    let mut current_time: i64 = 0;
    let mut ticks = Vec::new();
    let mut start_sn = src.sn;

    for i in 0..src.time.len() {
        current_time += src.time[i];
        let cur_price = (src.price_base + src.price[i]) as f64 / denominator;
        let mv: &trade_tick_data::MergedVol = &src.merged_vols[i];
        for (j, &vol) in mv.vol.iter().enumerate() {
            ticks.push(PushTick {
                // sn formula mirrors Java TradeTickUtil (startSn * 10 + j).
                // The server guarantees MergedVol.vol.len() < 10 per time-slot
                // (futures typically have 1–3 venues), so the multiplier of 10
                // is sufficient and consistent with the Java/Python/Go SDKs.
                sn: start_sn * 10 + j as i64,
                time: current_time,
                price: cur_price,
                volume: vol,
                tick_type: String::new(),
                cond: String::new(),
                part_code: String::new(),
                part_name: String::new(),
            });
        }
        start_sn += 1;
    }

    PushTradeTick {
        symbol: src.symbol,
        sec_type: src.sec_type,
        quote_level: String::new(),
        timestamp: src.timestamp,
        ticks,
    }
}
