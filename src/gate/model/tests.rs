use super::{Gate, Threshold, Verdict};
use crate::signal::{Level, Measure, Reading, SignalId, Trend};
use crate::units::{Bytes, ClockTime, ClockWindow, Count, Decimal};

fn load(thousandths: u64) -> Measure {
    Measure::Number(Decimal::from_thousandths(thousandths))
}

fn value(measure: Measure) -> Reading {
    Reading::Value(measure)
}

fn fails(verdict: &Verdict) -> bool {
    matches!(verdict, Verdict::Fail { .. })
}

fn unsupported(verdict: &Verdict) -> bool {
    matches!(verdict, Verdict::Unsupported { .. })
}

#[test]
fn below_passes_strictly_under_the_threshold() {
    let gate = Gate::below(SignalId::Load, load(2_000));
    assert_eq!(gate.judge(&value(load(1_999)), None), Verdict::Pass);
}

#[test]
fn below_fails_at_equality() {
    let gate = Gate::below(SignalId::Load, load(2_000));
    assert!(fails(&gate.judge(&value(load(2_000)), None)));
}

#[test]
fn below_fails_over_the_threshold_with_value_and_trend() {
    let gate = Gate::below(SignalId::Load, load(2_000));
    let verdict = gate.judge(&value(load(3_000)), Some(Trend::Rising));
    let fail = Verdict::Fail {
        value: load(3_000),
        threshold: Threshold::Value(load(2_000)),
        trend: Some(Trend::Rising),
    };
    assert_eq!(verdict, fail);
}

#[test]
fn above_passes_strictly_over_the_threshold() {
    let gate = Gate::above(SignalId::Mem, Measure::Bytes(Bytes(1_000)));
    let reading = value(Measure::Bytes(Bytes(1_001)));
    assert_eq!(gate.judge(&reading, None), Verdict::Pass);
}

#[test]
fn above_fails_at_equality() {
    let gate = Gate::above(SignalId::Mem, Measure::Bytes(Bytes(1_000)));
    let reading = value(Measure::Bytes(Bytes(1_000)));
    assert!(fails(&gate.judge(&reading, None)));
}

#[test]
fn per_cpu_divides_load_before_comparing() {
    let gate = Gate::below(SignalId::Load, load(1_000)).per_cpu(4);
    assert_eq!(gate.judge(&value(load(3_999)), None), Verdict::Pass);
    assert!(fails(&gate.judge(&value(load(4_000)), None)));
}

#[test]
fn per_cpu_divides_memory_before_comparing() {
    let gate = Gate::above(SignalId::Mem, Measure::Bytes(Bytes(100)));
    let gate = gate.per_cpu(4);
    let verdict = gate.judge(&value(Measure::Bytes(Bytes(400))), None);
    let shown = Measure::Bytes(Bytes(100));
    assert!(matches!(verdict, Verdict::Fail { value, .. } if value == shown));
}

#[test]
fn per_cpu_of_no_cpus_is_unsupported() {
    let gate = Gate::below(SignalId::Load, load(1_000)).per_cpu(0);
    assert!(unsupported(&gate.judge(&value(load(1)), None)));
}

#[test]
fn unreserved_takes_reservations_off_available_memory() {
    let gate = Gate::above(SignalId::Mem, Measure::Bytes(Bytes(500)));
    let gate = gate.unreserved(Bytes(600));
    let reading = value(Measure::Bytes(Bytes(1_000)));
    assert!(fails(&gate.judge(&reading, None)));
}

#[test]
fn is_passes_on_the_same_flag_only() {
    let gate = Gate::is(SignalId::OnAc, Measure::Flag(true));
    let on = gate.judge(&value(Measure::Flag(true)), None);
    assert_eq!(on, Verdict::Pass);
    assert!(fails(&gate.judge(&value(Measure::Flag(false)), None)));
}

#[test]
fn is_passes_a_level_at_most_the_limit() {
    let gate = Gate::is(SignalId::MemLevel, Measure::Level(Level::Warning));
    let normal = value(Measure::Level(Level::Normal));
    let critical = value(Measure::Level(Level::Critical));
    assert_eq!(gate.judge(&normal, None), Verdict::Pass);
    assert!(fails(&gate.judge(&critical, None)));
}

#[test]
fn within_passes_inside_a_window_that_wraps_midnight() {
    let at = |h, m| ClockTime::new(h, m);
    let window = at(22, 0)
        .zip(at(6, 0))
        .and_then(|(s, e)| ClockWindow::new(s, e));
    let night = at(23, 30).map(Measure::Clock).map(value);
    let noon = at(12, 0).map(Measure::Clock).map(value);
    let verdicts = window.zip(night.zip(noon)).map(|(w, (n, d))| {
        let gate = Gate::within(SignalId::Between, w);
        (gate.judge(&n, None), fails(&gate.judge(&d, None)))
    });
    assert_eq!(verdicts, Some((Verdict::Pass, true)));
}

#[test]
fn warming_reading_is_a_warming_verdict() {
    let gate = Gate::below(SignalId::CpuIdle, load(1));
    assert_eq!(gate.judge(&Reading::Warming, None), Verdict::Warming);
}

#[test]
fn unsupported_reading_carries_its_reason() {
    let gate = Gate::below(SignalId::CpuPressure, load(1));
    let verdict = gate.judge(&Reading::unsupported("no PSI on macOS"), None);
    let reason = match verdict {
        Verdict::Unsupported { reason } => reason.to_string(),
        _ => String::new(),
    };
    assert_eq!(reason, "no PSI on macOS");
}

#[test]
fn a_reading_in_another_unit_is_unsupported_never_a_pass() {
    let gate = Gate::below(SignalId::Load, load(1_000));
    let reading = value(Measure::Count(Count(0)));
    assert!(unsupported(&gate.judge(&reading, None)));
}

#[test]
fn a_window_against_a_number_is_unsupported() {
    let window = ClockTime::new(1, 0)
        .zip(ClockTime::new(2, 0))
        .and_then(|(s, e)| ClockWindow::new(s, e));
    let verdict = window.map(|w| {
        Gate::within(SignalId::Between, w).judge(&value(load(1)), None)
    });
    assert!(verdict.is_some_and(|v| unsupported(&v)));
}

#[test]
fn threshold_renders_value_or_window() {
    let window = ClockTime::new(22, 0)
        .zip(ClockTime::new(6, 0))
        .and_then(|(s, e)| ClockWindow::new(s, e));
    let shown = window.map(|w| Threshold::Window(w).to_string());
    assert_eq!(shown.as_deref(), Some("22:00-06:00"));
    assert_eq!(Threshold::Value(load(1_500)).to_string(), "1.5");
}

/// One low and one high value in every quantity unit but percent.
fn ordered_pairs() -> [(Measure, Measure); 4] {
    use crate::units::{Celsius, Rate};
    use std::time::Duration;
    let celsius = |t| Measure::Celsius(Celsius::from_tenths(t));
    let duration = Measure::Duration;
    [
        (Measure::Count(Count(1)), Measure::Count(Count(2))),
        (Measure::Rate(Rate(1)), Measure::Rate(Rate(2))),
        (duration(Duration::ZERO), duration(Duration::MAX)),
        (celsius(1), celsius(2)),
    ]
}

fn below_passes((low, high): (Measure, Measure)) -> bool {
    Gate::below(SignalId::Temp, high)
        .judge(&value(low), None)
        .is_pass()
}

#[test]
fn every_quantity_compares_in_its_own_unit() {
    use crate::units::Percent;
    let pct = |h| Percent::from_hundredths(h).map(Measure::Percent);
    assert!(ordered_pairs().into_iter().all(below_passes));
    assert_eq!(pct(100).zip(pct(200)).map(below_passes), Some(true));
}

#[test]
fn a_flag_is_not_a_quantity_to_order() {
    let gate = Gate::below(SignalId::Cool, Measure::Flag(true));
    assert!(unsupported(&gate.judge(&value(Measure::Flag(false)), None)));
}
