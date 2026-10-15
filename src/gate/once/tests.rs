use std::time::Duration;

use super::{evaluate_once, sample_once};
use crate::gate::{Gate, Gates, Overall};
use crate::signal::{
    FakeSource, ManualClock, Measure, Reading, SignalId, SignalSet,
};
use crate::units::{Count, Decimal, Percent};

const INTERVAL: Duration = Duration::from_secs(2);

fn idle(hundredths: u16) -> Reading {
    Percent::from_hundredths(hundredths)
        .map_or(Reading::Warming, |p| Reading::Value(Measure::Percent(p)))
}

fn load(thousandths: u64) -> Reading {
    Reading::Value(Measure::Number(Decimal::from_thousandths(thousandths)))
}

fn source() -> FakeSource {
    let mut source = FakeSource::new();
    source.script(SignalId::CpuIdle, [Reading::Warming, idle(5_000)]);
    source.script(SignalId::Load, [load(1)]);
    source
}

fn idle_gate() -> Gate {
    let floor = Percent::from_hundredths(1_000).map(Measure::Percent);
    Gate::above(SignalId::CpuIdle, floor.unwrap_or(Measure::Count(Count(0))))
}

fn load_gate() -> Gate {
    let ceiling = Measure::Number(Decimal::from_thousandths(2_000));
    Gate::below(SignalId::Load, ceiling)
}

#[test]
fn a_rate_gate_takes_a_second_sample_after_one_interval() {
    let (mut source, mut clock) = (source(), ManualClock::new());
    let mut gates = Gates::new();
    gates.push(idle_gate()).push(load_gate());
    let eval = evaluate_once(&mut gates, &mut source, &mut clock, INTERVAL);
    assert_eq!(eval.overall, Overall::Pass);
    assert_eq!(clock.sleeps(), &[INTERVAL]);
    assert_eq!(source.reads(SignalId::CpuIdle), 2);
    assert_eq!(source.reads(SignalId::Load), 1);
}

#[test]
fn gates_without_rates_read_once_and_never_wait() {
    let (mut source, mut clock) = (source(), ManualClock::new());
    let mut gates = Gates::new();
    gates.push(load_gate());
    let eval = evaluate_once(&mut gates, &mut source, &mut clock, INTERVAL);
    assert_eq!(eval.overall, Overall::Pass);
    assert!(clock.sleeps().is_empty());
    assert_eq!(source.reads(SignalId::Load), 1);
}

#[test]
fn an_own_source_rate_gate_is_primed_too() {
    let (mut shared, mut clock) = (FakeSource::new(), ManualClock::new());
    let mut gates = Gates::new();
    gates.push_own(idle_gate(), source());
    let eval = evaluate_once(&mut gates, &mut shared, &mut clock, INTERVAL);
    assert_eq!(eval.overall, Overall::Pass);
    assert_eq!(clock.sleeps(), &[INTERVAL]);
}

#[test]
fn status_reads_rates_twice_and_shows_no_warming() {
    let (mut source, mut clock) = (source(), ManualClock::new());
    let wanted: SignalSet =
        [SignalId::CpuIdle, SignalId::Load].into_iter().collect();
    let read = sample_once(&mut source, wanted, &mut clock, INTERVAL);
    assert!(read.iter().all(|(_, r)| *r != Reading::Warming));
    assert_eq!(clock.sleeps(), &[INTERVAL]);
    assert_eq!(read.len(), 2);
}

#[test]
fn status_without_rates_never_waits() {
    let (mut source, mut clock) = (source(), ManualClock::new());
    let wanted: SignalSet = [SignalId::Load].into_iter().collect();
    let read = sample_once(&mut source, wanted, &mut clock, INTERVAL);
    assert_eq!(read, vec![(SignalId::Load, load(1))]);
    assert!(clock.sleeps().is_empty());
}
