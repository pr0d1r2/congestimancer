use super::{Ran, ran};
use crate::cli::remote::ScriptedSsh;

/// A file that exists wherever the tests run.
const HERE: &str = "Cargo.toml";
/// A file that exists nowhere.
const NOWHERE: &str = "no/such/file/for/cgmr";
/// A gate no machine supports: `--cpu-throttle` read on the host; its
/// second sample waits only 10ms.
const UNSUPPORTED: [&str; 6] = [
    "--cgroup",
    "host",
    "--cpu-throttle",
    "50",
    "--interval",
    "10ms",
];

fn cgmr(args: &[&str]) -> Ran {
    with(args, &[])
}

fn with(args: &[&str], vars: &[(&str, &str)]) -> Ran {
    ran(args, vars, &mut ScriptedSsh::default())
}

#[test]
fn version_prints_the_crate_version() {
    let ran = cgmr(&["--version"]);
    assert_eq!(ran.code, 0);
    assert_eq!(
        ran.out.trim(),
        format!("cgmr {}", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(cgmr(&["check", "--version"]).code, 0);
}

#[test]
fn a_clap_error_is_a_usage_error() {
    let ran = cgmr(&["--load", "1", "--load", "2"]);
    assert_eq!(ran.code, 64);
    assert!(
        ran.err.contains("cannot be used multiple times"),
        "{}",
        ran.err
    );
    assert_eq!(cgmr(&["frobnicate"]).code, 64);
}

#[test]
fn a_bad_value_is_a_usage_error_naming_the_switch() {
    let ran = cgmr(&["check", "--load", "lots"]);
    assert_eq!(ran.code, 64);
    assert!(ran.err.starts_with("cgmr: --load: "), "{}", ran.err);
    let env = with(&["check"], &[("CGMR_DISK", "huge")]);
    assert!(env.err.starts_with("cgmr: CGMR_DISK: "), "{}", env.err);
}

#[test]
fn run_without_a_command_exits_0_once_the_gates_pass() {
    assert_eq!(cgmr(&[]).code, 0);
    assert_eq!(cgmr(&["--when-file", HERE]).code, 0);
    assert_eq!(cgmr(&["run", "--when", "true"]).code, 0);
}

#[test]
fn run_times_out_with_124() {
    let args = [
        "--when-file",
        NOWHERE,
        "--interval",
        "10ms",
        "--timeout",
        "30ms",
    ];
    let ran = cgmr(&args);
    assert_eq!(ran.code, 124);
    assert!(ran.err.contains("--timeout ran out"), "{}", ran.err);
}

#[test]
fn or_background_runs_anyway_after_the_timeout() {
    let args = ["--when-file", NOWHERE, "--interval", "10ms"];
    let ran =
        cgmr(&[&args[..], &["--timeout", "20ms", "--or-background"]].concat());
    assert_eq!(ran.code, 0);
}

#[test]
fn an_unsupported_gate_exits_69() {
    let ran = cgmr(&UNSUPPORTED);
    assert_eq!(ran.code, 69);
    assert!(ran.err.contains("--cpu-throttle"), "{}", ran.err);
    assert_eq!(cgmr(&[&["check"], &UNSUPPORTED[..]].concat()).code, 69);
}

#[test]
fn check_exits_0_on_pass_and_1_on_fail() {
    assert_eq!(cgmr(&["check", "--when-file", HERE]).code, 0);
    assert_eq!(cgmr(&["check", "--when-file", NOWHERE]).code, 1);
    assert_eq!(cgmr(&["check"]).code, 0);
}

#[test]
fn check_json_reports_the_gates() {
    let ran = cgmr(&["check", "--json", "--when-file", NOWHERE]);
    assert_eq!(ran.code, 1);
    assert!(ran.out.starts_with("{\"schema_version\":1"), "{}", ran.out);
    assert!(ran.out.contains("\"switch\":\"when-file\""), "{}", ran.out);
}

#[test]
fn check_verbose_says_what_blocks() {
    let ran = cgmr(&["check", "--verbose", "--when-file", NOWHERE]);
    assert!(ran.err.contains("--when-file"), "{}", ran.err);
}

#[test]
fn quiet_hides_warnings_but_not_errors() {
    let vars = [("CGMR_LODE", "1")];
    assert!(with(&["check"], &vars).err.contains("CGMR_LOAD"));
    assert_eq!(with(&["check", "--quiet"], &vars).err, "");
    let quiet = [("CGMR_LODE", "1"), ("CGMR_QUIET", "yes")];
    assert_eq!(with(&["check"], &quiet).err, "");
    let error = with(&["check", "--quiet", "--load", "x"], &vars);
    assert!(error.err.contains("--load"), "{}", error.err);
}

#[test]
fn the_environment_sets_a_gate() {
    let vars = [("CGMR_WHEN_FILE", NOWHERE)];
    assert_eq!(with(&["check"], &vars).code, 1);
    let flag = with(&["check", "--when-file", HERE], &vars);
    assert_eq!(flag.code, 0);
}

#[test]
fn cgmr_when_is_never_run() {
    let vars = [("CGMR_WHEN", "false")];
    let ran = with(&["check"], &vars);
    assert_eq!(ran.code, 0);
    assert!(ran.err.contains("CGMR_WHEN is ignored"), "{}", ran.err);
}

#[test]
fn the_process_entry_point_returns_an_exit_code() {
    // argv is the test runner's own, so only that it returns is checked.
    let code = super::cgmr();
    assert!(!format!("{code:?}").is_empty());
}
