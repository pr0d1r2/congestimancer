use super::{RemoteGate, RemoteReport, RemoteVerdict, parse_reply};
use crate::gate::{
    Checked, Custom, Gate, Overall, Subject, Threshold, Verdict,
};
use crate::report::{StatusReport, StatusRow, write_json};
use crate::signal::{Measure, Reading, SignalId, Trend};
use crate::units::{Bytes, Decimal};

const V1: &str = include_str!("../../../tests/fixtures/remote/check-v1.json");
const V2: &str = include_str!("../../../tests/fixtures/remote/check-v2.json");

fn load(thousandths: u64) -> Measure {
    Measure::Number(Decimal::from_thousandths(thousandths))
}

fn checked(subject: Subject, verdict: Verdict) -> Checked {
    Checked { subject, verdict }
}

fn local_gates() -> [Checked; 3] {
    let gib = Measure::Bytes(Bytes(1 << 30));
    let fail = Verdict::Fail {
        value: Measure::Bytes(Bytes(1)),
        threshold: Threshold::Value(gib),
        trend: None,
    };
    let load_gate = Gate::below(SignalId::Load, load(2_000));
    let when = Custom::When("test \"$x\"".into());
    [
        checked(Subject::Signal(load_gate), Verdict::Pass),
        checked(Subject::Signal(Gate::above(SignalId::Mem, gib)), fail),
        checked(Subject::Custom(when), Verdict::Warming),
    ]
}

fn local_report() -> StatusReport {
    let mut report = StatusReport::new("b\"ox");
    let row = StatusRow::new(SignalId::Load, Reading::Value(load(1_500)));
    report.signals.push(row.with_trend(Some(Trend::Falling)));
    let gone = Reading::unsupported("no GPU");
    report.signals.push(StatusRow::new(SignalId::Gpu, gone));
    report.gates.extend(local_gates());
    report
}

fn written(report: &StatusReport) -> String {
    let mut out = Vec::new();
    assert!(write_json(&mut out, report).is_ok());
    String::from_utf8(out).unwrap_or_default()
}

fn gate(switch: &str, threshold: &str, verdict: RemoteVerdict) -> RemoteGate {
    RemoteGate {
        switch: switch.to_owned(),
        threshold: threshold.to_owned(),
        verdict,
    }
}

#[test]
fn write_json_round_trips() {
    let parsed = parse_reply(&written(&local_report()));
    let report = parsed.unwrap_or_default();
    assert_eq!(report.host, "b\"ox");
    let want = vec![
        gate("all-load", "2", RemoteVerdict::Pass),
        gate("all-mem", "1GiB", RemoteVerdict::Fail),
        gate("when", "test \"$x\"", RemoteVerdict::Warming),
    ];
    assert_eq!(report.gates, want);
    assert_eq!(report.overall(), Overall::Fail);
}

#[test]
fn signals_round_trip() {
    let report = parse_reply(&written(&local_report())).unwrap_or_default();
    let load = report.signals.first();
    assert_eq!(load.map(|s| s.id.as_str()), Some("load"));
    assert_eq!(load.and_then(|s| s.value.as_deref()), Some("1.5"));
    assert_eq!(load.and_then(|s| s.unit.as_deref()), Some("number"));
    assert_eq!(load.and_then(|s| s.trend.as_deref()), Some("falling"));
    assert_eq!(load.map(|s| s.source.as_str()), Some("host"));
    let gpu = report.signals.get(1);
    assert_eq!(
        gpu.map(|s| (s.supported, s.value.clone())),
        Some((false, None))
    );
}

#[test]
fn the_fixture_parses_and_ignores_unknown_fields() {
    let report = parse_reply(V1).unwrap_or_default();
    assert_eq!(report.host, "box");
    assert_eq!(report.signals.len(), 2);
    assert_eq!(report.overall(), Overall::Unsupported);
}

#[test]
fn another_schema_version_is_exit_69() {
    let err = parse_reply(V2).err();
    assert_eq!(err.as_ref().map(|e| e.exit_code()), Some(69));
    let text = err.map(|e| e.to_string()).unwrap_or_default();
    assert!(
        text.contains("schema 2") && text.contains("schema 1"),
        "{text}"
    );
}

#[test]
fn a_missing_schema_version_is_exit_69() {
    let err = parse_reply("{\"host\":\"box\",\"signals\":[],\"gates\":[]}");
    assert_eq!(err.map_err(|e| e.exit_code()), Err(69));
}

#[test]
fn garbled_or_mistyped_replies_are_exit_69() {
    let bad = [
        "not json",
        "{\"schema_version\":1}",
        "{\"schema_version\":\"1\",\"host\":\"b\",\"signals\":[],\"gates\":[]}",
        "{\"schema_version\":1,\"host\":\"b\",\"signals\":[],\
         \"gates\":[{\"switch\":\"l\",\"threshold\":\"2\",\"verdict\":\"maybe\"}]}",
    ];
    for reply in bad {
        assert_eq!(
            parse_reply(reply).map_err(|e| e.exit_code()),
            Err(69),
            "{reply}"
        );
    }
}

#[test]
fn no_gates_pass() {
    assert_eq!(RemoteReport::default().overall(), Overall::Pass);
}

#[test]
fn verdicts_join_like_the_local_gate() {
    let mut report = RemoteReport::default();
    report.gates.push(gate("a", "1", RemoteVerdict::Warming));
    assert_eq!(report.overall(), Overall::Warming);
    report.gates.push(gate("b", "1", RemoteVerdict::Fail));
    assert_eq!(report.overall(), Overall::Fail);
    report.gates.push(gate("c", "1", RemoteVerdict::Pass));
    assert_eq!(report.overall(), Overall::Fail);
}

#[test]
fn block_reasons_name_each_gate_that_did_not_pass() {
    let report = parse_reply(V1).unwrap_or_default();
    let want = ["--load 2: fail on box", "--gpu 50%: unsupported on box"];
    assert_eq!(report.block_reasons(), want);
}
