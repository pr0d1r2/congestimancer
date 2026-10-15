use std::time::Instant;

use super::{write_advice, write_advice_json};
use crate::advise::{Advice, AdviseRequest, decide};
use crate::gate::{Checked, Gate, Gates, evaluate};
use crate::signal::{FakeSource, Measure, Reading, SignalId, Trends};
use crate::units::{Bytes, Count, Decimal};

const GIB: u64 = 1 << 30;

/// The verdict of an `--fds` gate below 100 reading `open`, or warming.
fn fds_gate(open: Reading) -> Vec<Checked> {
    let mut source = FakeSource::new();
    source.script(SignalId::Fds, [open]);
    let mut gates = Gates::new();
    gates.push(Gate::below(SignalId::Fds, Measure::Count(Count(100))));
    evaluate(&mut gates, &mut source, Instant::now(), &mut Trends::new())
        .verdicts
}

/// 8 CPUs at load 5, 2GiB available, 6 jobs asked: reduce to 2 by mem.
fn advice(gates: &[Checked]) -> Advice {
    let request = AdviseRequest::new(8).with_jobs(6);
    decide(
        &request,
        Decimal::from_thousandths(5_000),
        Bytes(2 * GIB),
        gates,
    )
}

fn human(advice: &Advice) -> String {
    let mut out = Vec::new();
    let _ = write_advice(&mut out, advice);
    String::from_utf8_lossy(&out).into_owned()
}

fn json(advice: &Advice) -> String {
    let mut out = Vec::new();
    let _ = write_advice_json(&mut out, advice);
    String::from_utf8_lossy(&out).into_owned()
}

#[test]
fn the_human_form_is_one_line() {
    assert_eq!(human(&advice(&[])), "jobs=2 limit=mem action=reduce\n");
}

#[test]
fn a_blocking_gate_names_itself_as_the_limit() {
    let gates = fds_gate(Reading::Value(Measure::Count(Count(200))));
    assert_eq!(human(&advice(&gates)), "jobs=0 limit=fds action=wait\n");
}

#[test]
fn json_shares_the_schema_version_and_has_no_blocking_gate() {
    assert_eq!(
        json(&advice(&[])),
        "{\"schema_version\":1,\"jobs\":2,\"limit\":\"mem\",\
         \"action\":\"reduce\",\"blocking\":[]}\n"
    );
}

#[test]
fn json_lists_blocking_gates_with_value_and_threshold() {
    let gates = fds_gate(Reading::Value(Measure::Count(Count(200))));
    assert_eq!(
        json(&advice(&gates)),
        "{\"schema_version\":1,\"jobs\":0,\"limit\":\"fds\",\
         \"action\":\"wait\",\"blocking\":[{\"switch\":\"fds\",\
         \"value\":\"200\",\"threshold\":\"100\",\"verdict\":\"fail\"}]}\n"
    );
}

#[test]
fn a_warming_gate_blocks_with_no_value() {
    let gates = fds_gate(Reading::Warming);
    let text = json(&advice(&gates));
    assert!(
        text.contains(
            "{\"switch\":\"fds\",\"value\":null,\"threshold\":\"100\",\
             \"verdict\":\"warming\"}"
        ),
        "{text}"
    );
}
