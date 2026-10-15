use std::path::PathBuf;
use std::time::Duration;

use super::{block_reasons, heartbeat_line, write_verbose};
use crate::gate::{
    Checked, Custom, Evaluation, Gate, Overall, Subject, Threshold, Verdict,
};
use crate::report::Style;
use crate::signal::{Level, Measure, SignalId, Trend};
use crate::units::{Bytes, ClockTime, ClockWindow, Decimal};

fn load(thousandths: u64) -> Measure {
    Measure::Number(Decimal::from_thousandths(thousandths))
}

fn fail(gate: Gate, value: Measure, trend: Option<Trend>) -> Checked {
    Checked {
        subject: Subject::Signal(gate),
        verdict: Verdict::Fail {
            value,
            threshold: gate.threshold,
            trend,
        },
    }
}

fn of(verdicts: Vec<Checked>) -> Evaluation {
    Evaluation {
        verdicts,
        overall: Overall::Fail,
    }
}

fn one(checked: Checked) -> String {
    block_reasons(&of(vec![checked])).join("|")
}

fn load_gate() -> Gate {
    Gate::below(SignalId::Load, load(2_000)).per_cpu(4)
}

#[test]
fn a_failing_gate_names_value_threshold_and_trend() {
    let checked = fail(load_gate(), load(3_000), Some(Trend::Rising));
    assert_eq!(one(checked), "--load: 3 (needs < 2, rising)");
}

#[test]
fn a_failure_without_a_trend_leaves_it_out() {
    let mem = Gate::above(SignalId::Mem, Measure::Bytes(Bytes(2)));
    let checked = fail(mem, Measure::Bytes(Bytes(1)), None);
    assert_eq!(one(checked), "--all-mem: 1B (needs > 2B)");
}

#[test]
fn a_level_needs_at_most_its_threshold() {
    let gate = Gate::is(SignalId::MemLevel, Measure::Level(Level::Warning));
    let checked = fail(gate, Measure::Level(Level::Critical), None);
    assert_eq!(one(checked), "--mem-level: critical (needs <= warning)");
}

#[test]
fn a_flag_or_window_needs_its_threshold_as_is() {
    let cool = Gate::is(SignalId::Cool, Measure::Flag(true));
    let checked = fail(cool, Measure::Flag(false), None);
    assert_eq!(one(checked), "--cool: no (needs yes)");
    let at = |h| ClockTime::new(h, 0);
    let window = at(22).zip(at(6)).and_then(|(s, e)| ClockWindow::new(s, e));
    let shown = window.zip(at(12)).map(|(w, noon)| {
        let gate = Gate::within(SignalId::Between, w);
        one(fail(gate, Measure::Clock(noon), None))
    });
    let want = "--between: 12:00 (needs 22:00-06:00)";
    assert_eq!(shown.as_deref(), Some(want));
}

#[test]
fn a_custom_gate_shows_its_command() {
    let checked = Checked {
        subject: Subject::Custom(Custom::When("test -f go".into())),
        verdict: Verdict::Fail {
            value: Measure::Flag(false),
            threshold: Threshold::Value(Measure::Flag(true)),
            trend: None,
        },
    };
    assert_eq!(one(checked), "--when test -f go: no (needs yes)");
}

#[test]
fn warming_and_unsupported_gates_say_why() {
    let gate = Subject::Signal(Gate::below(SignalId::IoBusy, load(1)));
    let warming = Checked {
        subject: gate.clone(),
        verdict: Verdict::Warming,
    };
    let unsupported = Checked {
        subject: gate,
        verdict: Verdict::Unsupported {
            reason: "no disk".into(),
        },
    };
    let shown = block_reasons(&of(vec![warming, unsupported]));
    let want = ["--io-busy: warming up", "--io-busy: unsupported: no disk"];
    assert_eq!(shown, want);
}

#[test]
fn passing_gates_are_not_reasons() {
    let file = Subject::Custom(Custom::WhenFile(PathBuf::from("go")));
    let pass = Checked {
        subject: file,
        verdict: Verdict::Pass,
    };
    assert!(block_reasons(&of(vec![pass])).is_empty());
}

#[test]
fn verbose_writes_one_blocked_line_per_reason() {
    let evaluation = of(vec![fail(load_gate(), load(3_000), None)]);
    let mut out = Vec::new();
    assert!(write_verbose(&mut out, &evaluation, Style::Plain).is_ok());
    assert_eq!(out, b"blocked: --load: 3 (needs < 2)\n");
}

#[test]
fn verbose_paints_the_label_red() {
    let evaluation = of(vec![fail(load_gate(), load(3_000), None)]);
    let mut out = Vec::new();
    assert!(write_verbose(&mut out, &evaluation, Style::Ansi).is_ok());
    let want = "\u{1b}[31mblocked:\u{1b}[0m --load: 3 (needs < 2)\n";
    assert_eq!(String::from_utf8(out).unwrap_or_default(), want);
}

#[test]
fn heartbeat_names_the_wait_and_every_blocker() {
    let evaluation = of(vec![
        fail(load_gate(), load(3_000), Some(Trend::Falling)),
        Checked {
            subject: Subject::Signal(Gate::below(SignalId::IoBusy, load(1))),
            verdict: Verdict::Warming,
        },
    ]);
    let line = heartbeat_line(&evaluation, Duration::from_secs(90));
    let want = "cgmr: waiting 1m30s: --load: 3 (needs < 2, falling); \
                --io-busy: warming up";
    assert_eq!(line, want);
}

#[test]
fn heartbeat_with_nothing_blocking_says_the_gates_pass() {
    let evaluation = Evaluation {
        verdicts: Vec::new(),
        overall: Overall::Pass,
    };
    let line = heartbeat_line(&evaluation, Duration::from_secs(5));
    assert_eq!(line, "cgmr: waiting 5s: all gates pass");
}
