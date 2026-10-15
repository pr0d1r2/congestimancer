use std::path::PathBuf;

use super::{StatusReport, StatusRow, switch_name};
use crate::gate::{Custom, Gate, Subject};
use crate::signal::{Measure, Reading, SignalId, Trend};
use crate::units::{Bytes, Decimal};

fn load(thousandths: u64) -> Measure {
    Measure::Number(Decimal::from_thousandths(thousandths))
}

fn mem() -> Gate {
    Gate::above(SignalId::Mem, Measure::Bytes(Bytes(1)))
}

fn signal(gate: Gate) -> String {
    switch_name(&Subject::Signal(gate))
}

#[test]
fn a_row_reads_the_host_with_no_trend_by_default() {
    let row = StatusRow::new(SignalId::Load, Reading::Warming);
    assert_eq!(row.source, "host");
    assert_eq!(row.trend, None);
}

#[test]
fn a_row_takes_a_trend_and_a_source() {
    let row = StatusRow::new(SignalId::Load, Reading::Warming)
        .with_trend(Some(Trend::Rising))
        .with_source("cgroup");
    assert_eq!(row.source, "cgroup");
    assert_eq!(row.trend, Some(Trend::Rising));
}

#[test]
fn a_report_starts_empty_on_its_host() {
    let report = StatusReport::new("box");
    assert_eq!(report.host, "box");
    assert!(report.signals.is_empty() && report.gates.is_empty());
}

#[test]
fn per_cpu_gates_keep_the_plain_switch() {
    let gate = Gate::below(SignalId::Load, load(1)).per_cpu(4);
    assert_eq!(signal(gate), "load");
    assert_eq!(signal(mem().per_cpu(4)), "mem");
}

#[test]
fn whole_gates_on_per_cpu_signals_are_the_all_switches() {
    assert_eq!(signal(Gate::below(SignalId::Load, load(1))), "all-load");
    let queue = Gate::below(SignalId::RunQueue, load(1));
    assert_eq!(signal(queue), "all-run-queue");
    assert_eq!(signal(mem()), "all-mem");
}

#[test]
fn unreserved_memory_is_needs_mem() {
    assert_eq!(signal(mem().unreserved(Bytes(5))), "needs-mem");
}

#[test]
fn other_signals_are_their_own_switch() {
    assert_eq!(signal(Gate::below(SignalId::IoBusy, load(1))), "io-busy");
}

#[test]
fn custom_gates_are_when_and_when_file() {
    let when = Subject::Custom(Custom::When("true".into()));
    assert_eq!(switch_name(&when), "when");
    let file = Subject::Custom(Custom::WhenFile(PathBuf::from("/x")));
    assert_eq!(switch_name(&file), "when-file");
}
