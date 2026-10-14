use super::{Trend, Trends};
use crate::signal::{Measure, Reading, SignalId};
use crate::units::Count;

/// A reading of `n` things.
fn count(n: u64) -> Reading {
    Reading::Value(Measure::Count(Count(n)))
}

/// The trend of one signal after `readings`.
fn after(readings: &[Reading]) -> Option<Trend> {
    let mut trends = Trends::new();
    for r in readings {
        trends.record(SignalId::IoQueue, r);
    }
    trends.trend(SignalId::IoQueue)
}

#[test]
fn a_trend_compares_two_numbers() {
    assert_eq!(Trend::between(1.0, 2.0), Trend::Rising);
    assert_eq!(Trend::between(2.0, 1.0), Trend::Falling);
    assert_eq!(Trend::between(1.5, 1.5), Trend::Flat);
    assert_eq!(Trend::between(f64::NAN, 1.0), Trend::Flat);
}

#[test]
fn a_trend_needs_two_values() {
    assert_eq!(after(&[]), None);
    assert_eq!(after(&[count(1)]), None);
    assert_eq!(after(&[count(1), count(2)]), Some(Trend::Rising));
}

#[test]
fn only_the_last_two_values_count() {
    let got = after(&[count(9), count(1), count(1)]);
    assert_eq!(got, Some(Trend::Flat));
    let down = after(&[count(1), count(5), count(3)]);
    assert_eq!(down, Some(Trend::Falling));
}

#[test]
fn readings_without_a_value_leave_the_history() {
    let gap = [count(1), Reading::Warming, Reading::unsupported("x")];
    assert_eq!(after(&gap), None);
    let got = after(&[count(1), Reading::Warming, count(3)]);
    assert_eq!(got, Some(Trend::Rising));
}

#[test]
fn every_signal_has_its_own_trend() {
    let mut trends = Trends::new();
    trends.record(SignalId::Load, &count(1));
    trends.record(SignalId::Mem, &count(5));
    trends.record(SignalId::Load, &count(2));
    assert_eq!(trends.trend(SignalId::Load), Some(Trend::Rising));
    assert_eq!(trends.trend(SignalId::Mem), None);
}

#[test]
fn a_trend_renders_as_a_word() {
    let words =
        [Trend::Rising, Trend::Falling, Trend::Flat].map(|t| t.to_string());
    assert_eq!(words, ["rising", "falling", "flat"]);
}
