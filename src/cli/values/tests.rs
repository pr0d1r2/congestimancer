use super::parse_level;
use crate::cli::table::Value;
use crate::signal::{Level, Measure};
use crate::units::{Bytes, Count};

#[test]
fn level_takes_normal_or_warning() {
    assert_eq!(parse_level("normal"), Ok(Level::Normal));
    assert_eq!(parse_level("warning"), Ok(Level::Warning));
    assert!(parse_level("critical").is_err());
}

#[test]
fn measures_parse_in_their_unit() {
    let gib = Value::Bytes.measure("1GiB");
    assert_eq!(gib, Some(Ok(Measure::Bytes(Bytes(1 << 30)))));
    assert_eq!(
        Value::Count.measure("3"),
        Some(Ok(Measure::Count(Count(3))))
    );
    let level = Value::Level.measure("warning");
    assert_eq!(level, Some(Ok(Measure::Level(Level::Warning))));
    assert!(Value::Path.measure("/tmp").is_none());
}

#[test]
fn a_bad_measure_names_the_input() {
    let bad = Value::Percent.measure("lots");
    assert!(matches!(bad, Some(Err(e)) if e.contains("lots")));
}

/// One good text of each value kind that is no measure.
const GOOD: [(Value, &str); 14] = [
    (Value::Decimal, "1.5"),
    (Value::Rate, "50MB/s"),
    (Value::Duration, "1h30m"),
    (Value::Celsius, "80C"),
    (Value::Window, "22:00-06:00"),
    (Value::LoadWindow, "5m"),
    (Value::Device, "sda"),
    (Value::Iface, "en0"),
    (Value::Host, "builder"),
    (Value::Cgroup, "auto"),
    (Value::ProcLimit, "rustc*=4"),
    (Value::StartRate, "4/1m"),
    (Value::Name, "build"),
    (Value::Path, "/tmp"),
];

#[test]
fn every_value_kind_checks_its_text() {
    for (value, text) in GOOD {
        assert_eq!(value.check(text), Ok(()), "{value:?} {text}");
    }
}

#[test]
fn every_value_kind_refuses_garbage() {
    let bad = [
        Value::Decimal,
        Value::Window,
        Value::LoadWindow,
        Value::Host,
        Value::Cgroup,
        Value::ProcLimit,
        Value::StartRate,
        Value::Name,
    ];
    for value in bad {
        assert!(value.check("-?").is_err(), "{value:?}");
    }
    assert!(Value::Text.check(" ").is_err());
}
