use std::time::Duration;

use super::{AdviseUnsupported, advise, advise_exit_code, advise_readings};
use crate::advise::{AdviceAction, AdviceLimit, AdviseRequest};
use crate::gate::{Gate, Gates};
use crate::signal::{
    FakeSource, ManualClock, Measure, Reading, Reason, SignalId,
};
use crate::units::{Bytes, Count, Decimal, Percent};

const GIB: u64 = 1 << 30;
const INTERVAL: Duration = Duration::from_secs(3);

fn load(thousandths: u64) -> Reading {
    Reading::Value(Measure::Number(Decimal::from_thousandths(thousandths)))
}

fn mem(gib: u64) -> Reading {
    Reading::Value(Measure::Bytes(Bytes(gib.saturating_mul(GIB))))
}

fn idle(hundredths: u16) -> Reading {
    Percent::from_hundredths(hundredths)
        .map_or(Reading::Warming, |p| Reading::Value(Measure::Percent(p)))
}

fn idle_gate(hundredths: u16) -> Gate {
    let floor = Percent::from_hundredths(hundredths).map(Measure::Percent);
    Gate::above(SignalId::CpuIdle, floor.unwrap_or(Measure::Count(Count(0))))
}

/// Load 5 and 10GiB available: 8 CPUs fit 3 jobs.
fn machine() -> FakeSource {
    let mut source = FakeSource::new();
    source.script(SignalId::Load, [load(5_000)]);
    source.script(SignalId::Mem, [mem(10)]);
    source.script(SignalId::CpuIdle, [Reading::Warming, idle(5_000)]);
    source
}

fn unsupported(why: &'static str) -> Reading {
    Reading::unsupported(why)
}

#[test]
fn readings_advise_from_load_and_memory() {
    let advice =
        advise_readings(&AdviseRequest::new(8), &load(5_000), &mem(1), &[]);
    let advice = advice.map(|a| (a.jobs, a.limit, a.action));
    assert_eq!(advice, Ok((1, AdviceLimit::Mem, AdviceAction::Run)));
}

#[test]
fn an_unsupported_load_is_unsupported_on_the_load_switch() {
    let got = advise_readings(
        &AdviseRequest::new(8),
        &unsupported("no load here"),
        &mem(1),
        &[],
    );
    let err = got.err().map(|e| (e.switch, e.reason));
    assert_eq!(err, Some(("load".to_owned(), Reason::from("no load here"))));
}

#[test]
fn an_unsupported_memory_is_unsupported_on_needs_mem() {
    let got = advise_readings(
        &AdviseRequest::new(8),
        &load(0),
        &unsupported("no memory here"),
        &[],
    );
    assert_eq!(got.err().map(|e| e.switch), Some("needs-mem".to_owned()));
}

#[test]
fn a_reading_in_the_wrong_unit_is_unsupported() {
    let got = advise_readings(&AdviseRequest::new(8), &mem(1), &mem(1), &[]);
    assert_eq!(got.err().map(|e| e.switch), Some("load".to_owned()));
    let got = advise_readings(&AdviseRequest::new(8), &load(0), &load(0), &[]);
    assert_eq!(got.err().map(|e| e.switch), Some("needs-mem".to_owned()));
}

#[test]
fn warming_load_or_memory_fits_nothing() {
    let request = AdviseRequest::new(8).with_interval(INTERVAL);
    let cold_load = advise_readings(&request, &Reading::Warming, &mem(10), &[]);
    let cold_mem = advise_readings(&request, &load(0), &Reading::Warming, &[]);
    let jobs =
        |r: Result<_, AdviseUnsupported>| r.map(|a: super::Advice| a.jobs);
    assert_eq!(jobs(cold_load), Ok(0));
    assert_eq!(jobs(cold_mem), Ok(0));
}

#[test]
fn the_verb_takes_one_extra_sample_for_a_rate_gate() {
    let (mut source, mut clock) = (machine(), ManualClock::new());
    let mut gates = Gates::new();
    gates.push(idle_gate(1_000));
    let request = AdviseRequest::new(8).with_interval(INTERVAL);
    let got = advise(&request, &mut gates, &mut source, &mut clock);
    assert_eq!(got.as_ref().map(|a| a.jobs), Ok(3));
    assert_eq!(clock.sleeps(), &[INTERVAL]);
    assert_eq!(source.reads(SignalId::Load), 1);
    assert_eq!(source.reads(SignalId::Mem), 1);
    assert_eq!(advise_exit_code(&got), 0);
}

#[test]
fn the_verb_never_waits_without_a_rate_gate() {
    let (mut source, mut clock) = (machine(), ManualClock::new());
    let request = AdviseRequest::new(8).with_interval(INTERVAL);
    let mut gates = Gates::new();
    let got = advise(&request, &mut gates, &mut source, &mut clock);
    assert_eq!(got.map(|a| a.action), Ok(AdviceAction::Run));
    assert!(clock.sleeps().is_empty());
}

#[test]
fn a_failing_gate_advises_wait_and_exits_zero() {
    let (mut source, mut clock) = (machine(), ManualClock::new());
    let mut gates = Gates::new();
    gates.push(idle_gate(9_000));
    let request = AdviseRequest::new(8).with_interval(INTERVAL);
    let got = advise(&request, &mut gates, &mut source, &mut clock);
    let limit = AdviceLimit::Gate("cpu-idle".to_owned());
    let seen = got.as_ref().map(|a| (a.jobs, &a.limit, a.action));
    assert_eq!(seen, Ok((0, &limit, AdviceAction::Wait)));
    assert_eq!(advise_exit_code(&got), 0);
}

#[test]
fn an_unsupported_gate_exits_69() {
    let (mut source, mut clock) = (machine(), ManualClock::new());
    let mut gates = Gates::new();
    gates.push(Gate::below(SignalId::Fds, Measure::Count(Count(10))));
    let request = AdviseRequest::new(8).with_interval(INTERVAL);
    let got = advise(&request, &mut gates, &mut source, &mut clock);
    assert_eq!(got.as_ref().err().map(|e| e.switch.as_str()), Some("fds"));
    assert_eq!(advise_exit_code(&got), AdviseUnsupported::EXIT);
}

#[test]
fn unsupported_renders_the_switch_and_reason() {
    let err = AdviseUnsupported {
        switch: "load".to_owned(),
        reason: Reason::from("no load here"),
    };
    assert_eq!(err.to_string(), "--load: unsupported: no load here");
}
