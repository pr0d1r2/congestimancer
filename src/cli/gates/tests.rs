use super::{Fit, build};
use crate::cli::exit::USAGE;
use crate::cli::layers::{Layer, Resolved};
use crate::gate::{Adjust, Gate, Op, Threshold};
use crate::signal::{Measure, SignalId};
use crate::units::{Bytes, Decimal};

const FIT: Fit = Fit {
    cpus: 4,
    reserved: Bytes(10),
};

fn set(pairs: &[(&'static str, &[&str])]) -> Resolved {
    let mut resolved = Resolved::default();
    for (name, values) in pairs {
        resolved = resolved.with(name, values, Layer::Flag);
    }
    resolved
}

/// The debug text of the gates `pairs` build.
fn gates(pairs: &[(&'static str, &[&str])], skip: &[&str]) -> String {
    let built = build(&set(pairs), FIT, skip);
    built.map(|b| format!("{:?}", b.gates)).unwrap_or_default()
}

#[test]
fn a_per_cpu_gate_is_adjusted() {
    let want = Gate::below(
        SignalId::Load,
        Measure::Number(Decimal::from_thousandths(2_000)),
    )
    .per_cpu(4);
    assert_eq!(want.adjust, Adjust::PerCpu(4));
    assert_eq!(gates(&[("load", &["2"])], &[]), format!("[{want:?}]"));
}

#[test]
fn needs_mem_is_less_the_reservations() {
    let text = gates(&[("needs-mem", &["1GB"])], &[]);
    assert!(text.contains("Unreserved(Bytes(10))"), "{text}");
    assert_eq!(gates(&[("needs-mem", &["1GB"])], &["needs-mem"]), "[]");
}

#[test]
fn flags_windows_and_levels_are_states() {
    let text =
        gates(&[("cool", &["true"]), ("between", &["22:00-06:00"])], &[]);
    assert!(text.contains("Cool"), "{text}");
    assert!(text.contains("Window"), "{text}");
    let off = gates(&[("cool", &["false"])], &[]);
    assert_eq!(off, "[]");
    let level = gates(&[("mem-level", &["warning"])], &[]);
    assert!(level.contains("Level(Warning)"), "{level}");
}

#[test]
fn each_process_limit_is_its_own_gate() {
    let built = build(&set(&[("max-procs", &["a=1", "b=2"])]), FIT, &[]);
    assert_eq!(built.as_ref().map(|b| b.gates.len()).ok(), Some(2));
    let signals = built.map(|b| b.signals.is_empty()).ok();
    assert_eq!(signals, Some(true));
    let users = build(&set(&[("max-user-procs", &["nixbld*=8"])]), FIT, &[]);
    assert_eq!(users.map(|b| b.gates.len()).ok(), Some(1));
}

#[test]
fn custom_gates_are_built() {
    let text = gates(&[("when", &["true"]), ("when-file", &["/x"])], &[]);
    assert!(text.contains("When(\"true\")"), "{text}");
    assert!(text.contains("WhenFile(\"/x\")"), "{text}");
}

#[test]
fn shared_gates_record_their_signals() {
    let built =
        build(&set(&[("disk", &["1GB"]), ("all-load", &["3"])]), FIT, &[]);
    let signals = built.map(|b| b.signals).unwrap_or_default();
    assert!(signals.contains(SignalId::Disk));
    assert!(signals.contains(SignalId::Load));
}

#[test]
fn a_bad_value_is_a_usage_error() {
    let refused = build(&set(&[("max-procs", &["nope"])]), FIT, &[]);
    assert_eq!(refused.err().map(|f| f.code), Some(USAGE));
    let bad = build(&set(&[("between", &["noon"])]), FIT, &[]);
    assert_eq!(bad.err().map(|f| f.code), Some(USAGE));
}

#[test]
fn above_gates_compare_upward() {
    let built = build(&set(&[("disk", &["1GB"])]), FIT, &[]);
    let text = built.map(|b| format!("{:?}", b.gates)).unwrap_or_default();
    assert!(text.contains(&format!("{:?}", Op::Above)), "{text}");
    let threshold = Threshold::Value(Measure::Bytes(Bytes(1_000_000_000)));
    assert!(text.contains(&format!("{threshold:?}")), "{text}");
}
