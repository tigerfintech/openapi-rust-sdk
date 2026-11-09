use prost::Message;
use tigeropen::model::quote_requests::OptionKlineItem;
use tigeropen::model::{OptionBrief, OptionHourTrading};
use tigeropen::push::pb;

// Fixtures generated from stock-openapi feat_gth commit 33df7142f.
#[test]
fn server_gth_wire_preserves_flat_fields_and_optional_zeros() {
    for (hex, price, time, volume, amount) in [
        ("0a185350585720323032363037323320373338302e3020505554100118a4fb9df1f83331000000000000f83f3889fa9df1f833499a9999999999f13f500e5900000000000029407a095072654d61726b6574e9010000000000001c40f001d00ff901000000008087c340820203475448", 1.5, 1784796380425, 7, 12.5),
        ("0a185350585720323032363037323320373338302e3020505554100118a4fb9df1f8333100000000000000003800499a9999999999f13f50005900000000000000007a095072654d61726b6574e9010000000000000000f001d00ff901000000008087c340820203475448", 0.0, 0, 0, 0.0),
    ] {
        let bytes: Vec<u8> = (0..hex.len()).step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i+2], 16).unwrap()).collect();
        let q = pb::QuoteData::decode(bytes.as_slice()).unwrap();
        let basic = pb::QuoteBasicData::decode(bytes.as_slice()).unwrap();
        assert_eq!(q.latest_price, Some(price));
        assert_eq!(q.latest_price_timestamp, Some(time));
        assert_eq!(q.volume, Some(volume));
        assert_eq!(q.volume_decimal, Some(volume as f64));
        assert_eq!(q.amount, Some(amount));
        assert_eq!(q.cum_volume, Some(1000));
        assert_eq!(q.cum_amount, Some(9999.0));
        assert_eq!(q.pre_close, Some(1.1));
        assert_eq!(q.hour_trading_tag.as_deref(), Some("PreMarket"));
        assert_eq!(q.session.as_deref(), Some("GTH"));
        assert_eq!(basic.latest_price, q.latest_price);
        assert_eq!(basic.latest_price_timestamp, q.latest_price_timestamp);
        assert_eq!(basic.volume, q.volume);
        assert_eq!(basic.volume_decimal, q.volume_decimal);
        assert_eq!(basic.amount, q.amount);
        assert_eq!(basic.cum_volume, q.cum_volume);
        assert_eq!(basic.cum_amount, q.cum_amount);
        assert_eq!(basic.pre_close, q.pre_close);
        assert_eq!(basic.hour_trading_tag, q.hour_trading_tag);
        assert_eq!(basic.session, q.session);
    }
    let regular = pb::QuoteData::decode([0x10, 0x01].as_slice()).unwrap();
    assert!(regular.latest_price.is_none());
    assert!(regular.latest_price_timestamp.is_none());
    assert!(regular.volume.is_none());
    assert!(regular.volume_decimal.is_none());
    assert!(regular.amount.is_none());
    assert!(regular.cum_volume.is_none());
    assert!(regular.cum_amount.is_none());
    assert!(regular.session.is_none());
}

#[test]
fn gth_http_fields_preserve_null_and_zero() {
    let q: OptionBrief = serde_json::from_str(r#"{"symbol":"SPXW","cumVolume":0,"hourTrading":{"session":"GTH","latestTimestamp":1784796380425,"volume":0}}"#).unwrap();
    assert_eq!(q.cum_volume, Some(0));
    let h: OptionHourTrading = q.hour_trading.unwrap();
    assert_eq!(h.latest_timestamp, Some(1784796380425));
    assert_eq!(h.volume, Some(0));
    let regular: OptionBrief = serde_json::from_str(r#"{"hourTrading":null}"#).unwrap();
    assert!(regular.hour_trading.is_none());
}

#[test]
fn gth_kline_flag_omits_none_and_keeps_false() {
    for flag in [None, Some(true), Some(false)] {
        let mut item = OptionKlineItem::from_occ("SPXW  260723P07380000", "5min").unwrap();
        item.with_hour_trading = flag;
        let body = serde_json::to_value(item).unwrap();
        match flag {
            None => assert!(body.get("with_hour_trading").is_none()),
            Some(value) => assert_eq!(body["with_hour_trading"], value),
        }
    }
}
