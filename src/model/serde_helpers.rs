//! Lenient deserializers for Tiger API quirks.
//!
//! Tiger's wire format is *mostly* numeric epoch-millisecond timestamps,
//! but some endpoints occasionally return human-readable date strings
//! ("YYYY-MM-DD HH:MM:SS", no timezone) — observed in practice from
//! `/order_transactions` for paper accounts. Without a permissive
//! deserializer the whole response decode fails on the first such row.
//!
//! `deserialize_lenient_timestamp` accepts any of:
//!   - JSON number: passed through (or `f64 → i64` truncation if needed)
//!   - JSON null: `0`
//!   - String of digits ("1747353034000"): parsed as i64
//!   - Naive datetime "YYYY-MM-DD HH:MM:SS": parsed as **UTC**, returned
//!     as epoch millis. Tiger does not document the actual timezone for
//!     these strings; UTC is a defensible default for ordering /
//!     display since the rest of Tiger's epoch fields are also UTC.
//!   - RFC 3339 ("2026-05-08T22:57:14Z" or with offset): parsed
//!
//! Anything else (object, array, garbled string) deserializes to `0`
//! and emits a `tracing::warn!`. The fallback keeps a single bad row
//! from poisoning a whole `Vec<T>` decode.
//!
//! Naive datetimes are converted via `NaiveDateTime::and_utc()`
//! (chrono 0.4.31+ — already required transitively by tigeropen's
//! existing chrono dep).

use serde::{Deserialize, Deserializer};
use serde_json::Value;

pub(crate) fn deserialize_lenient_timestamp<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(parse_lenient_timestamp(&value))
}

fn parse_lenient_timestamp(value: &Value) -> i64 {
    match value {
        Value::Null => 0,
        Value::Number(n) => n
            .as_i64()
            .unwrap_or_else(|| n.as_f64().map(|f| f as i64).unwrap_or(0)),
        Value::String(s) => {
            let s = s.trim();
            if s.is_empty() {
                return 0;
            }
            if let Ok(n) = s.parse::<i64>() {
                return n;
            }
            if let Ok(dt) =
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
            {
                return dt.and_utc().timestamp_millis();
            }
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return dt.timestamp_millis();
            }
            tracing::warn!(
                value = %s,
                "lenient_timestamp: unrecognized string shape; defaulting to 0"
            );
            0
        }
        other => {
            tracing::warn!(
                ?other,
                "lenient_timestamp: unexpected JSON shape; defaulting to 0"
            );
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq)]
    struct Wrap {
        #[serde(deserialize_with = "deserialize_lenient_timestamp")]
        ts: i64,
    }

    fn parse(s: &str) -> i64 {
        serde_json::from_str::<Wrap>(s).unwrap().ts
    }

    #[test]
    fn integer_passes_through() {
        assert_eq!(parse(r#"{"ts": 1700000000000}"#), 1_700_000_000_000);
        assert_eq!(parse(r#"{"ts": 0}"#), 0);
        assert_eq!(parse(r#"{"ts": -1}"#), -1);
    }

    #[test]
    fn float_truncates_to_i64() {
        // serde_json may emit i64 as f64 in some configs.
        assert_eq!(parse(r#"{"ts": 1700000000000.0}"#), 1_700_000_000_000);
        assert_eq!(parse(r#"{"ts": 1700000000000.5}"#), 1_700_000_000_000);
    }

    #[test]
    fn null_becomes_zero() {
        assert_eq!(parse(r#"{"ts": null}"#), 0);
    }

    #[test]
    fn numeric_string_parses() {
        assert_eq!(parse(r#"{"ts": "1700000000000"}"#), 1_700_000_000_000);
        assert_eq!(parse(r#"{"ts": "  1700000000000  "}"#), 1_700_000_000_000);
    }

    #[test]
    fn naive_datetime_parses_as_utc() {
        // The exact string observed from Tiger paper:
        // "2026-05-08 22:57:14"
        // 2026-05-08 22:57:14 UTC ≡ 1778281034 epoch seconds
        //                         = 1_778_281_034_000 epoch millis
        let got = parse(r#"{"ts": "2026-05-08 22:57:14"}"#);
        assert_eq!(got, 1_778_281_034_000);
    }

    #[test]
    fn rfc3339_z_parses() {
        assert_eq!(
            parse(r#"{"ts": "2026-05-08T22:57:14Z"}"#),
            1_778_281_034_000
        );
    }

    #[test]
    fn rfc3339_with_offset_normalizes_to_utc() {
        // 2026-05-08 22:57:14 +08:00 == 2026-05-08 14:57:14 UTC
        // 14:57:14 = 53834 s into the day; 20581*86400 + 53834 = 1778252234
        assert_eq!(
            parse(r#"{"ts": "2026-05-08T22:57:14+08:00"}"#),
            1_778_252_234_000
        );
    }

    #[test]
    fn empty_string_becomes_zero() {
        assert_eq!(parse(r#"{"ts": ""}"#), 0);
        assert_eq!(parse(r#"{"ts": "   "}"#), 0);
    }

    #[test]
    fn unknown_string_falls_back_to_zero() {
        assert_eq!(parse(r#"{"ts": "not a date"}"#), 0);
        assert_eq!(parse(r#"{"ts": "2026-13-45"}"#), 0);
    }

    #[test]
    fn unexpected_shape_falls_back_to_zero() {
        assert_eq!(parse(r#"{"ts": []}"#), 0);
        assert_eq!(parse(r#"{"ts": {}}"#), 0);
        assert_eq!(parse(r#"{"ts": true}"#), 0);
    }
}
