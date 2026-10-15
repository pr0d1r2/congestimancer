use std::time::Instant;

use super::{Gates, Overall, Subject, evaluate};
use crate::gate::{Gate, Verdict};
use crate::signal::{FakeSource, Measure, Reading, SignalId, Trend, Trends};
use crate::units::{Bytes, Decimal};

fn load(thousandths: u64) -> Measure {
    Measure::Number(Decimal::from_thousandths(thousandths))
}

fn bytes(n: u64) -> Measure {
    Measure::Bytes(Bytes(n))
}

fn source(load_now: Reading, mem_now: Reading) -> FakeSource {
    let mut source = FakeSource::new();
    source.script(SignalId::Load, [load_now]);
    source.script(SignalId::Mem, [mem_now]);
    source
}

fn two_gates() -> Gates {
    let mut gates = Gates::new();
    gates.push(Gate::below(SignalId::Load, load(2_000)));
    gates.push(Gate::above(SignalId::Mem, bytes(100)));
    gates
}

fn overall(gates: &mut Gates, source: &mut FakeSource) -> Overall {
    let mut trends = Trends::new();
    evaluate(gates, source, Instant::now(), &mut trends).overall
}

#[test]
fn no_gates_pass_at_once() {
    let mut source = FakeSource::new();
    assert_eq!(overall(&mut Gates::new(), &mut source), Overall::Pass);
}

#[test]
fn all_gates_passing_pass() {
    let mut source =
        source(Reading::Value(load(1)), Reading::Value(bytes(101)));
    assert_eq!(overall(&mut two_gates(), &mut source), Overall::Pass);
}

#[test]
fn one_failing_gate_fails_the_whole() {
    let mut source =
        source(Reading::Value(load(1)), Reading::Value(bytes(100)));
    assert_eq!(overall(&mut two_gates(), &mut source), Overall::Fail);
}

#[test]
fn a_warming_gate_never_passes_yet() {
    let mut source = source(Reading::Warming, Reading::Value(bytes(101)));
    assert_eq!(overall(&mut two_gates(), &mut source), Overall::Warming);
}

#[test]
fn a_failure_outranks_warming() {
    let mut source = source(Reading::Warming, Reading::Value(bytes(1)));
    assert_eq!(overall(&mut two_gates(), &mut source), Overall::Fail);
}

#[test]
fn an_unsupported_gate_outranks_everything() {
    let no_psi = Reading::unsupported("no PSI on macOS");
    let mut source = source(no_psi, Reading::Value(bytes(1)));
    assert_eq!(overall(&mut two_gates(), &mut source), Overall::Unsupported);
}

#[test]
fn verdicts_follow_the_order_of_the_gates() {
    let mut source = source(Reading::Value(load(1)), Reading::Value(bytes(1)));
    let mut trends = Trends::new();
    let now = Instant::now();
    let eval = evaluate(&mut two_gates(), &mut source, now, &mut trends);
    let subjects: Vec<_> = eval.verdicts.iter().map(|c| &c.subject).collect();
    let mem = Subject::Signal(Gate::above(SignalId::Mem, bytes(100)));
    assert_eq!(subjects.last(), Some(&&mem));
    assert!(eval.verdicts.first().is_some_and(|c| c.verdict.is_pass()));
}

#[test]
fn several_gates_on_one_signal_read_it_once_per_round() {
    let mut gates = Gates::new();
    gates.push(Gate::below(SignalId::Load, load(2_000)));
    gates.push(Gate::below(SignalId::Load, load(1_000)).per_cpu(4));
    let mut source = source(Reading::Value(load(1)), Reading::Warming);
    let _ = overall(&mut gates, &mut source);
    assert_eq!(source.reads(SignalId::Load), 1);
    assert_eq!(source.reads(SignalId::Mem), 0);
}

#[test]
fn an_own_source_gate_reads_its_own_source() {
    let mut own = FakeSource::new();
    own.script(SignalId::Load, [Reading::Value(load(1))]);
    let mut gates = Gates::new();
    gates.push_own(Gate::below(SignalId::Load, load(2_000)), own);
    let mut shared = FakeSource::new();
    assert_eq!(overall(&mut gates, &mut shared), Overall::Pass);
    assert_eq!(shared.reads(SignalId::Load), 0);
}

#[test]
fn a_failure_carries_the_trend_across_rounds() {
    let mut source = FakeSource::new();
    let rising = [load(3_000), load(4_000)].map(Reading::Value);
    source.script(SignalId::Load, rising);
    let mut gates = Gates::new();
    gates.push(Gate::below(SignalId::Load, load(2_000)));
    let mut trends = Trends::new();
    let now = Instant::now();
    let _ = evaluate(&mut gates, &mut source, now, &mut trends);
    let eval = evaluate(&mut gates, &mut source, now, &mut trends);
    let trend = eval.verdicts.first().and_then(|c| match c.verdict {
        Verdict::Fail { trend, .. } => trend,
        _ => None,
    });
    assert_eq!(trend, Some(Trend::Rising));
}

#[test]
fn gates_debug_lists_the_gates() {
    let gates = two_gates();
    assert_eq!(gates.len(), 2);
    assert!(!gates.is_empty());
    assert!(format!("{gates:?}").contains("Mem"));
}

#[test]
fn only_a_pass_lets_the_command_run() {
    assert!(Overall::Pass.is_pass());
    assert!(!Overall::Warming.is_pass());
}
