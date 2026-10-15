use std::path::PathBuf;
use std::time::Duration;

use super::{SCHEMA_VERSION, json_string, write_json};
use crate::gate::{Checked, Custom, Gate, Subject, Threshold, Verdict};
use crate::report::{StatusReport, StatusRow};
use crate::signal::{Level, Measure, Reading, SignalId, Trend};
use crate::units::{Bytes, Celsius, ClockTime, Count, Decimal, Percent, Rate};

fn load(thousandths: u64) -> Measure {
    Measure::Number(Decimal::from_thousandths(thousandths))
}

fn json(report: &StatusReport) -> String {
    let mut out = Vec::new();
    assert!(write_json(&mut out, report).is_ok());
    String::from_utf8(out).unwrap_or_default()
}

fn signal_json(reading: Reading) -> String {
    let mut report = StatusReport::new("h");
    report.signals.push(StatusRow::new(SignalId::Load, reading));
    json(&report)
}

fn value_and_unit(measure: Measure) -> String {
    let all = signal_json(Reading::Value(measure));
    let from = all.find("\"value\":").unwrap_or_default();
    let to = all.find(",\"trend\"").unwrap_or_default();
    all.get(from..to).unwrap_or_default().to_owned()
}

#[test]
fn the_schema_version_is_one() {
    assert_eq!(SCHEMA_VERSION, 1);
}

#[test]
fn an_empty_report_has_the_version_host_and_empty_lists() {
    let want = "{\"schema_version\":1,\"host\":\"box\",\"signals\":[],\
                \"gates\":[]}\n";
    assert_eq!(json(&StatusReport::new("box")), want);
}

#[test]
fn a_signal_has_every_stable_field() {
    let mut report = StatusReport::new("h");
    let row = StatusRow::new(SignalId::IoBusy, Reading::Value(load(1_500)))
        .with_trend(Some(Trend::Rising))
        .with_source("cgroup");
    report.signals.push(row);
    let want = "{\"schema_version\":1,\"host\":\"h\",\"signals\":[\
                {\"id\":\"io-busy\",\"value\":1.5,\"unit\":\"number\",\
                \"trend\":\"rising\",\"supported\":true,\
                \"source\":\"cgroup\"}],\"gates\":[]}\n";
    assert_eq!(json(&report), want);
}

#[test]
fn signals_are_separated_by_commas() {
    let mut report = StatusReport::new("h");
    report
        .signals
        .push(StatusRow::new(SignalId::Load, Reading::Warming));
    report
        .signals
        .push(StatusRow::new(SignalId::Mem, Reading::Warming));
    let all = json(&report);
    assert!(all.contains("\"source\":\"host\"},{\"id\":\"mem\""));
}

#[test]
fn an_unsupported_signal_has_no_value_and_is_not_supported() {
    let all = signal_json(Reading::unsupported("no PSI"));
    assert!(all.contains("\"value\":null,\"unit\":null,\"trend\":null"));
    assert!(all.contains("\"supported\":false"));
}

#[test]
fn a_warming_signal_is_supported_with_no_value() {
    let all = signal_json(Reading::Warming);
    assert!(all.contains("\"value\":null,\"unit\":null"));
    assert!(all.contains("\"supported\":true"));
}

#[test]
fn whole_quantities_are_integers_in_their_unit() {
    let big = u64::MAX;
    let bytes = value_and_unit(Measure::Bytes(Bytes(big)));
    assert_eq!(bytes, format!("\"value\":{big},\"unit\":\"bytes\""));
    let rate = value_and_unit(Measure::Rate(Rate(5)));
    assert_eq!(rate, "\"value\":5,\"unit\":\"bytes/s\"");
    let count = value_and_unit(Measure::Count(Count(7)));
    assert_eq!(count, "\"value\":7,\"unit\":\"count\"");
}

#[test]
fn fractions_use_a_dot() {
    let percent = Percent::from_hundredths(1_250).map(Measure::Percent);
    let shown = percent.map(value_and_unit).unwrap_or_default();
    assert_eq!(shown, "\"value\":12.5,\"unit\":\"percent\"");
    let hot = value_and_unit(Measure::Celsius(Celsius::from_tenths(725)));
    assert_eq!(hot, "\"value\":72.5,\"unit\":\"celsius\"");
    let whole = value_and_unit(load(2_000));
    assert_eq!(whole, "\"value\":2,\"unit\":\"number\"");
}

#[test]
fn durations_are_seconds() {
    let d = Measure::Duration(Duration::from_millis(1_500));
    assert_eq!(value_and_unit(d), "\"value\":1.5,\"unit\":\"seconds\"");
}

#[test]
fn flags_levels_and_times_keep_their_form() {
    let flag = value_and_unit(Measure::Flag(true));
    assert_eq!(flag, "\"value\":true,\"unit\":\"flag\"");
    let level = value_and_unit(Measure::Level(Level::Warning));
    assert_eq!(level, "\"value\":\"warning\",\"unit\":\"level\"");
    let time = ClockTime::new(22, 5).map(Measure::Clock);
    let shown = time.map(value_and_unit).unwrap_or_default();
    assert_eq!(shown, "\"value\":\"22:05\",\"unit\":\"clock\"");
}

fn gates_json(subject: &Subject, verdicts: Vec<Verdict>) -> String {
    let mut report = StatusReport::new("h");
    for verdict in verdicts {
        report.gates.push(Checked {
            subject: subject.clone(),
            verdict,
        });
    }
    json(&report)
}

#[test]
fn a_gate_has_its_switch_threshold_and_verdict() {
    let gate = Gate::below(SignalId::Load, load(2_000)).per_cpu(4);
    let fail = Verdict::Fail {
        value: load(3_000),
        threshold: Threshold::Value(load(2_000)),
        trend: None,
    };
    let unsupported = Verdict::Unsupported { reason: "x".into() };
    let verdicts = vec![Verdict::Pass, fail, Verdict::Warming, unsupported];
    let all = gates_json(&Subject::Signal(gate), verdicts);
    let one = "{\"switch\":\"load\",\"threshold\":\"2\",\"verdict\":";
    let want = format!(
        "\"gates\":[{one}\"pass\"}},{one}\"fail\"}},{one}\"warming\"}},\
         {one}\"unsupported\"}}]}}\n"
    );
    assert!(all.ends_with(&want));
}

#[test]
fn a_when_gate_has_its_command_as_threshold() {
    let when = Subject::Custom(Custom::When("test \"$X\"".into()));
    let all = gates_json(&when, vec![Verdict::Pass]);
    let want = "{\"switch\":\"when\",\"threshold\":\"test \\\"$X\\\"\"";
    assert!(all.contains(want));
}

#[test]
fn a_when_file_gate_has_its_path_as_threshold() {
    let file = Custom::WhenFile(PathBuf::from("/tmp/go"));
    let all = gates_json(&Subject::Custom(file), vec![Verdict::Pass]);
    let want = "{\"switch\":\"when-file\",\"threshold\":\"/tmp/go\"";
    assert!(all.contains(want));
}

#[test]
fn the_host_is_escaped() {
    let all = json(&StatusReport::new("a\"b"));
    assert!(all.contains("\"host\":\"a\\\"b\""));
}

#[test]
fn strings_are_quoted() {
    assert_eq!(json_string("plain"), "\"plain\"");
}

#[test]
fn quotes_and_backslashes_are_escaped() {
    assert_eq!(json_string("a\"b\\c"), "\"a\\\"b\\\\c\"");
}

#[test]
fn common_control_characters_use_short_escapes() {
    let shown = json_string("\n\r\t\u{8}\u{c}");
    assert_eq!(shown, "\"\\n\\r\\t\\b\\f\"");
}

#[test]
fn other_control_characters_use_unicode_escapes() {
    assert_eq!(json_string("\u{0}\u{1f}"), "\"\\u0000\\u001f\"");
}

#[test]
fn non_ascii_passes_through_as_utf8() {
    assert_eq!(
        json_string("za\u{17c}\u{f3}\u{142}\u{107} \u{65e5}\u{672c} \u{7f}"),
        "\"za\u{17c}\u{f3}\u{142}\u{107} \u{65e5}\u{672c} \u{7f}\""
    );
}
