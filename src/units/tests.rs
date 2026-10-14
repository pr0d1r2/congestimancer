//! Rules that hold across every value a switch takes.

use std::time::Duration;

use proptest::prelude::*;

use super::{
    Bytes, Celsius, ClockTime, ClockWindow, Count, Decimal, Percent, Rate,
    UnitError, parse_bytes, parse_celsius, parse_clock_window, parse_count,
    parse_decimal, parse_duration, parse_name, parse_percent, parse_rate,
    render_duration,
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

/// Sizes and how they render.
const SIZES: [(u64, &str); 11] = [
    (0, "0B"),
    (1, "1B"),
    (1000, "1KB"),
    (1024, "1KiB"),
    (1536, "1536B"),
    (1_024_000, "1000KiB"),
    (1_000_000_000, "1GB"),
    (1_073_741_824, "1GiB"),
    (1_000_000_000_000, "1TB"),
    (1_099_511_627_776, "1TiB"),
    (u64::MAX, "18446744073709551615B"),
];

#[test]
fn sizes_render_in_the_largest_exact_unit() {
    for (n, s) in SIZES {
        assert_eq!(Bytes(n).to_string(), s);
    }
    assert_eq!(Rate(50_000_000).to_string(), "50MB/s");
}

#[test]
fn percents_and_temperatures_render_without_trailing_zeros() {
    let percent = |h| Percent::from_hundredths(h).map(|p| p.to_string());
    let shown = [percent(1250), percent(8000), percent(1)];
    let want = ["12.5%", "80%", "0.01%"].map(|s| Some(s.to_owned()));
    assert_eq!(shown, want);
    assert_eq!(Celsius::from_tenths(725).to_string(), "72.5C");
    assert_eq!(Celsius::from_tenths(850).to_string(), "85C");
}

#[test]
fn decimals_render_without_trailing_zeros() {
    let decimal = |t| Decimal::from_thousandths(t).to_string();
    assert_eq!([decimal(500), decimal(4000)], ["0.5", "4"]);
    assert_eq!([decimal(12_125), decimal(10)], ["12.125", "0.01"]);
}

#[test]
fn counts_names_and_clocks_render_as_written() {
    assert_eq!(Count(4).to_string(), "4");
    let name = parse_name("build-1.v2").map(|n| n.to_string());
    assert_eq!(name.as_deref(), Ok("build-1.v2"));
    let night = parse_clock_window("22:00-06:00").map(|w| w.to_string());
    assert_eq!(night.as_deref(), Ok("22:00-06:00"));
    let t = ClockTime::new(9, 5).map(|t| t.to_string());
    assert_eq!(t.as_deref(), Some("09:05"));
}

#[test]
fn durations_render_largest_unit_first() {
    let shown = [
        (Duration::from_secs(5400), "1h30m"),
        (Duration::ZERO, "0s"),
        (Duration::from_millis(500), "500ms"),
        (Duration::from_millis(93_784_005), "1d2h3m4s5ms"),
        (Duration::from_nanos(1500), "0s"),
        (Duration::from_micros(1_000_500), "1s"),
    ];
    for (d, s) in shown {
        assert_eq!(render_duration(d), s, "{d:?}");
    }
}

/// Any clock window: two different times of day.
fn clock_window() -> impl Strategy<Value = ClockWindow> {
    (0_u8..24, 0_u8..60, 0_u8..24, 0_u8..60).prop_filter_map(
        "start equals end",
        |(h1, m1, h2, m2)| {
            ClockWindow::new(ClockTime::new(h1, m1)?, ClockTime::new(h2, m2)?)
        },
    )
}

/// Any name: the safe set, never a leading `-`, never `.` or `..`.
fn name() -> impl Strategy<Value = String> {
    "[A-Za-z0-9._][A-Za-z0-9._-]{0,63}"
        .prop_filter("a dot path", |s| s != "." && s != "..")
}

/// Cases per property; no regression files, since parsers are pure and a
/// failure prints its minimal input.
fn config() -> ProptestConfig {
    ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    }
}

/// Text near the grammars: digits, separators, signs and unit letters, so
/// cases reach deep into each parser rather than failing on the first
/// character.
const NEAR: &str = "[0-9.,:%/ +_-]{0,12}[a-zA-Z]{0,4}[0-9.:/-]{0,12}";

/// Runs every parser over `s`; the property is that none panics.
fn parse_all(s: &str) {
    let _ = (parse_bytes(s), parse_rate(s), parse_percent(s));
    let _ = (parse_duration(s), parse_celsius(s), parse_clock_window(s));
    let _ = (parse_count(s), parse_decimal(s), parse_name(s));
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn no_parser_panics_on_any_text(s in any::<String>()) {
        parse_all(&s);
    }

    #[test]
    fn no_parser_panics_near_its_grammar(s in NEAR) {
        parse_all(&s);
    }

    #[test]
    fn sizes_and_rates_round_trip(n in any::<u64>()) {
        prop_assert_eq!(parse_bytes(&Bytes(n).to_string()), Ok(Bytes(n)));
        prop_assert_eq!(parse_rate(&Rate(n).to_string()), Ok(Rate(n)));
    }

    #[test]
    fn percents_round_trip(h in 0_u16..=10_000) {
        let p = Percent::from_hundredths(h);
        prop_assert_eq!(p.map(|p| parse_percent(&p.to_string())), p.map(Ok));
    }

    #[test]
    fn temperatures_round_trip(t in any::<u32>()) {
        let c = Celsius::from_tenths(t);
        prop_assert_eq!(parse_celsius(&c.to_string()), Ok(c));
    }

    #[test]
    fn decimals_and_counts_round_trip(n in any::<u64>()) {
        let d = Decimal::from_thousandths(n);
        prop_assert_eq!(parse_decimal(&d.to_string()), Ok(d));
        prop_assert_eq!(parse_count(&Count(n).to_string()), Ok(Count(n)));
    }

    #[test]
    fn durations_round_trip(ms in any::<u64>()) {
        let d = Duration::from_millis(ms);
        prop_assert_eq!(parse_duration(&render_duration(d)), Ok(d));
    }

    #[test]
    fn clock_windows_round_trip(w in clock_window()) {
        prop_assert_eq!(parse_clock_window(&w.to_string()), Ok(w));
    }

    #[test]
    fn names_round_trip(s in name()) {
        let parsed = parse_name(&s);
        let again = parsed.as_ref().map(|n| parse_name(&n.to_string()));
        prop_assert_eq!(again, parsed.as_ref().map(|n| Ok(n.clone())));
        prop_assert_eq!(parsed.map(|n| n.as_str().to_owned()), Ok(s));
    }
}
