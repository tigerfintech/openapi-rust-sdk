//! tick_util 单元测试

use tigeropen::push::pb::{self, trade_tick_data};
use tigeropen::push::tick_util::{
    convert_trade_tick, get_part_name_by_code, get_part_short_name_by_code,
};

fn make_stock_data() -> pb::TradeTickData {
    pb::TradeTickData {
        symbol: "AAPL".into(),
        sec_type: "STK".into(),
        quote_level: "L1".into(),
        timestamp: 1700000000,
        sn: 100,
        price_base: 1500000,
        price_offset: 4,
        time: vec![1000, 500, 300],
        price: vec![0, 100, -50],
        volume: vec![200, 100, 150],
        part_code: vec!["n".into(), "t".into(), "p".into()],
        cond: "B T".into(),
        r#type: "+-*".into(),
        merged_vols: vec![],
    }
}

#[test]
fn test_get_part_short_name_known() {
    assert_eq!(get_part_short_name_by_code("n"), "NYSE");
    assert_eq!(get_part_short_name_by_code("t"), "NSDQ");
    assert_eq!(get_part_short_name_by_code("p"), "ARCA");
    assert_eq!(get_part_short_name_by_code("z"), "BZX");
    assert_eq!(get_part_short_name_by_code("a"), "AMEX");
}

#[test]
fn test_get_part_short_name_unknown() {
    assert_eq!(get_part_short_name_by_code("?"), "?");
    assert_eq!(get_part_short_name_by_code(""), "");
}

#[test]
fn test_get_part_name_known() {
    assert_eq!(
        get_part_name_by_code("n"),
        "New York Stock Exchange, LLC (NYSE)"
    );
    assert_eq!(
        get_part_name_by_code("t"),
        "NASDAQ Stock Market, LLC (NASDAQ)"
    );
}

#[test]
fn test_get_part_name_unknown() {
    assert_eq!(get_part_name_by_code("?"), "?");
}

#[test]
fn test_convert_stock_cumulative_time() {
    let out = convert_trade_tick(make_stock_data());
    assert_eq!(out.ticks[0].time, 1000);
    assert_eq!(out.ticks[1].time, 1500);
    assert_eq!(out.ticks[2].time, 1800);
}

#[test]
fn test_convert_stock_price() {
    let out = convert_trade_tick(make_stock_data());
    let want0 = (1500000 + 0) as f64 / 10000.0;
    let want1 = (1500000 + 100) as f64 / 10000.0;
    assert!((out.ticks[0].price - want0).abs() < 1e-9);
    assert!((out.ticks[1].price - want1).abs() < 1e-9);
}

#[test]
fn test_convert_stock_part_code() {
    let out = convert_trade_tick(make_stock_data());
    assert_eq!(out.ticks[0].part_code, "NYSE");
    assert_eq!(
        out.ticks[0].part_name,
        "New York Stock Exchange, LLC (NYSE)"
    );
    assert_eq!(out.ticks[1].part_code, "NSDQ");
    assert_eq!(out.ticks[2].part_code, "ARCA");
}

#[test]
fn test_convert_stock_us_cond() {
    let out = convert_trade_tick(make_stock_data());
    assert_eq!(out.ticks[0].cond, "US_BUNCHED_TRADE"); // 'B'
    assert_eq!(out.ticks[1].cond, "US_REGULAR_SALE"); // ' '
}

#[test]
fn test_convert_stock_sn() {
    let out = convert_trade_tick(make_stock_data());
    assert_eq!(out.ticks[0].sn, 100);
    assert_eq!(out.ticks[2].sn, 102);
}

#[test]
fn test_convert_stock_metadata() {
    let out = convert_trade_tick(make_stock_data());
    assert_eq!(out.symbol, "AAPL");
    assert_eq!(out.sec_type, "STK");
    assert_eq!(out.quote_level, "L1");
    assert_eq!(out.timestamp, 1700000000);
}

#[test]
fn test_convert_hk_cond() {
    let src = pb::TradeTickData {
        symbol: "00700.HK".into(),
        sec_type: "STK".into(),
        sn: 1,
        price_base: 3000000,
        price_offset: 4,
        time: vec![1000],
        price: vec![0],
        volume: vec![100],
        cond: "U".into(),
        ..Default::default()
    };
    let out = convert_trade_tick(src);
    assert_eq!(out.ticks[0].cond, "HK_AUCTION_TRADE");
}

#[test]
fn test_convert_future_merged_vols() {
    let src = pb::TradeTickData {
        symbol: "ES2312".into(),
        sec_type: "FUT".into(),
        timestamp: 1700000000,
        sn: 10,
        price_base: 45000000,
        price_offset: 4,
        time: vec![1000, 500],
        price: vec![0, 100],
        merged_vols: vec![
            trade_tick_data::MergedVol {
                merge_times: 2,
                vol: vec![100, 200],
            },
            trade_tick_data::MergedVol {
                merge_times: 1,
                vol: vec![300],
            },
        ],
        ..Default::default()
    };
    let out = convert_trade_tick(src);
    assert_eq!(out.ticks.len(), 3);
    assert_eq!(out.ticks[0].sn, 100); // 10*10+0
    assert_eq!(out.ticks[1].sn, 101); // 10*10+1
    assert_eq!(out.ticks[2].sn, 110); // 11*10+0
    assert_eq!(out.ticks[0].volume, 100);
    assert_eq!(out.ticks[2].volume, 300);
    assert_eq!(out.ticks[0].part_code, "");
}

#[test]
fn test_is_us_stock_symbol() {
    use tigeropen::push::is_us_stock_symbol;
    assert!(is_us_stock_symbol("AAPL"));
    assert!(is_us_stock_symbol("TSLA"));
    // HK numeric symbols must NOT be treated as US
    assert!(!is_us_stock_symbol("00700"));
    assert!(!is_us_stock_symbol("09988"));
    assert!(!is_us_stock_symbol(""));
}

#[test]
fn test_hk_cond_conversion() {
    let src = pb::TradeTickData {
        symbol: "00700".into(),
        sec_type: "STK".into(),
        quote_level: "L1".into(),
        timestamp: 1700000000,
        sn: 1,
        price_base: 30000,
        price_offset: 2,
        time: vec![500],
        price: vec![0],
        volume: vec![1000],
        cond: " ".into(),
        r#type: "+".into(),
        ..Default::default()
    };
    let out = convert_trade_tick(src);
    assert_eq!(out.ticks[0].cond, "HK_AUTOMATCH_NORMAL");
}
