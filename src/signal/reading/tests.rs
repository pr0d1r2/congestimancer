use std::time::Duration;

use super::{Level, Measure, Reading};
use crate::units::{Bytes, Celsius, ClockTime, Count, Decimal, Percent, Rate};

/// A percent known to be in range.
fn percent(hundredths: u16) -> Percent {
    Percent::from_hundredths(hundredths).unwrap_or_default()
}

/// A time of day known to be valid.
fn clock(hour: u8, minute: u8) -> ClockTime {
    ClockTime::new(hour, minute).unwrap_or_else(|| unreachable!())
}

/// One measure of each unit and how it renders.
fn rendered() -> [(Measure, &'static str); 11] {
    [
        (Measure::Number(Decimal::from_thousandths(1500)), "1.5"),
        (Measure::Count(Count(4)), "4"),
        (Measure::Percent(percent(1250)), "12.5%"),
        (Measure::Bytes(Bytes(1024)), "1KiB"),
        (Measure::Rate(Rate(50_000_000)), "50MB/s"),
        (Measure::Duration(Duration::from_millis(1500)), "1s500ms"),
        (Measure::Celsius(Celsius::from_tenths(725)), "72.5C"),
        (Measure::Flag(true), "yes"),
        (Measure::Flag(false), "no"),
        (Measure::Level(Level::Warning), "warning"),
        (Measure::Clock(clock(22, 0)), "22:00"),
    ]
}

#[test]
fn a_measure_renders_with_its_unit() {
    for (measure, text) in rendered() {
        assert_eq!(measure.to_string(), text, "{measure:?}");
        assert_eq!(Reading::Value(measure).to_string(), text);
    }
}

/// One measure of each unit and the plain number it stands for.
fn plain() -> [(Measure, f64); 10] {
    [
        (Measure::Number(Decimal::from_thousandths(1500)), 1.5),
        (Measure::Celsius(Celsius::from_tenths(725)), 72.5),
        (Measure::Count(Count(4)), 4.0),
        (Measure::Percent(percent(1250)), 12.5),
        (Measure::Bytes(Bytes(1024)), 1024.0),
        (Measure::Rate(Rate(10)), 10.0),
        (Measure::Duration(Duration::from_millis(1500)), 1.5),
        (Measure::Flag(true), 1.0),
        (Measure::Level(Level::Critical), 2.0),
        (Measure::Clock(clock(1, 30)), 90.0),
    ]
}

#[test]
fn a_measure_is_a_plain_number_for_trends() {
    for (measure, number) in plain() {
        assert!((measure.as_f64() - number).abs() < 1e-9, "{measure:?}");
    }
}

#[test]
fn levels_rank_from_normal_to_critical() {
    assert!(Level::Normal < Level::Warning);
    assert!(Level::Warning < Level::Critical);
    assert_eq!(Level::Critical.to_string(), "critical");
    assert_eq!(Level::Normal.to_string(), "normal");
}

#[test]
fn unsupported_carries_its_reason() {
    let r = Reading::unsupported("no PSI on macOS");
    let Reading::Unsupported { reason } = &r else {
        unreachable!("{r:?}")
    };
    assert_eq!(reason.as_str(), "no PSI on macOS");
    assert_eq!(r.to_string(), "unsupported: no PSI on macOS");
    let owned = Reading::unsupported(format!("no device {}", "sda"));
    assert_eq!(owned.to_string(), "unsupported: no device sda");
}

#[test]
fn only_a_value_has_a_measure() {
    let m = Measure::Count(Count(1));
    assert_eq!(Reading::Value(m).measure(), Some(&m));
    assert_eq!(Reading::Warming.measure(), None);
    assert_eq!(Reading::unsupported("x").measure(), None);
    assert_eq!(Reading::Warming.to_string(), "warming up");
}
