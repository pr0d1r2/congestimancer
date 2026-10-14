//! Rules that hold across every value a switch takes.

use super::{
    UnitError, parse_bytes, parse_celsius, parse_count, parse_decimal,
    parse_duration, parse_percent, parse_rate,
};

/// Whether a parse reported its input as negative.
fn negative<T>(parsed: Result<T, UnitError>) -> bool {
    matches!(parsed, Err(UnitError::Negative { .. }))
}

#[test]
fn no_threshold_is_negative() {
    let refused = [
        negative(parse_bytes("-1GB")),
        negative(parse_rate("-1MB/s")),
        negative(parse_percent("-1%")),
        negative(parse_celsius("-1C")),
        negative(parse_count("-1")),
        negative(parse_decimal("-0.5")),
        negative(parse_duration("-1s")),
    ];
    assert_eq!(refused, [true; 7]);
}

#[test]
fn load_like_values_take_decimals_and_counts_do_not() {
    assert!(parse_decimal("0.5").is_ok());
    assert!(parse_count("0.5").is_err());
    assert!(parse_count("4").is_ok());
}
