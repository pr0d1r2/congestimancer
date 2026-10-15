use std::time::Duration;

use super::{interval, row, shown};
use crate::cli::entry::{Ran, ran};
use crate::cli::layers::{Layer, Resolved};
use crate::cli::remote::ScriptedSsh;
use crate::signal::{Reading, SignalId};

/// A fixture tree laid out like `/`.
const FIXTURE: &str = "tests/fixtures/cpu/linux";

fn cgmr(args: &[&str]) -> Ran {
    let vars = [("CGMR_FIXTURE_ROOT", FIXTURE)];
    ran(args, &vars, &mut ScriptedSsh::default())
}

#[test]
fn interval_defaults_to_two_seconds() {
    assert_eq!(
        interval(&Resolved::default()).ok(),
        Some(Duration::from_secs(2))
    );
    let set = Resolved::default().with("interval", &["50ms"], Layer::Flag);
    assert_eq!(interval(&set).ok(), Some(Duration::from_millis(50)));
}

#[test]
fn status_shows_all_but_the_pattern_and_probe_signals() {
    let plain = shown(&Resolved::default());
    assert!(plain.contains(SignalId::Load));
    assert!(!plain.contains(SignalId::MaxProcs));
    assert!(!plain.contains(SignalId::Rtt));
    let probe = Resolved::default().with("rtt-host", &["h"], Layer::Flag);
    assert!(shown(&probe).contains(SignalId::Rtt));
}

#[test]
fn a_cgroup_signal_row_names_its_source() {
    let reading = Reading::Warming;
    assert_eq!(row(SignalId::Mem, reading.clone(), true).source, "cgroup");
    assert_eq!(row(SignalId::Load, reading.clone(), true).source, "host");
    assert_eq!(row(SignalId::Mem, reading, false).source, "host");
}

#[test]
fn status_prints_a_table_or_json() {
    let table = cgmr(&["status", "--interval", "10ms"]);
    assert_eq!(table.code, 0);
    assert!(table.out.contains("SIGNAL"), "{}", table.out);
    let json = cgmr(&["status", "--json", "--interval", "10ms"]);
    assert!(json.out.contains("\"signals\":[{"), "{}", json.out);
}

#[test]
fn advise_prints_the_advice() {
    let line = cgmr(&["advise", "--interval", "10ms", "--jobs", "1"]);
    assert_eq!(line.code, 0, "{}", line.err);
    assert!(line.out.starts_with("jobs="), "{}", line.out);
    let json = cgmr(&["advise", "--json", "--interval", "10ms"]);
    assert!(json.out.contains("\"jobs\":"), "{}", json.out);
}

#[test]
fn advise_exits_69_for_an_unsupported_gate() {
    let args = ["advise", "--interval", "10ms", "--cgroup", "host"];
    let ran = cgmr(&[&args[..], &["--cpu-throttle", "5"]].concat());
    assert_eq!(ran.code, 69);
    assert!(ran.err.contains("--cpu-throttle"), "{}", ran.err);
}

#[test]
fn a_bad_selector_is_a_usage_error() {
    let ran = cgmr(&["check", "--disk", "1GB", "--disk-path", "no/such/dir"]);
    assert_eq!(ran.code, 64);
    let cgroup = cgmr(&["check", "--cgroup", "/no/such/cgroup"]);
    assert_eq!(cgroup.code, 64, "{}", cgroup.err);
}
