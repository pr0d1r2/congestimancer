use std::path::{Path, PathBuf};

use super::{Custom, run};
use crate::gate::Verdict;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/gate")
        .join(name)
}

fn fails(verdict: &Verdict) -> bool {
    matches!(verdict, Verdict::Fail { .. })
}

#[test]
fn when_passes_on_exit_zero() {
    let gate = Custom::When("exit 0".to_owned());
    assert_eq!(gate.check(), Verdict::Pass);
}

#[test]
fn when_fails_on_any_other_exit() {
    let gate = Custom::When("exit 3".to_owned());
    assert!(fails(&gate.check()));
}

#[test]
fn when_runs_the_command_through_the_shell() {
    let gate = Custom::When("test 1 -lt 2 && true".to_owned());
    assert_eq!(gate.check(), Verdict::Pass);
}

#[test]
fn when_writes_its_output_to_stderr_and_still_passes() {
    let gate = Custom::When("echo to-stderr; echo also >&2".to_owned());
    assert_eq!(gate.check(), Verdict::Pass);
}

#[test]
fn a_shell_that_cannot_start_is_unsupported() {
    let verdict = run("cgmr-no-such-shell", "exit 0");
    assert!(matches!(verdict, Verdict::Unsupported { .. }));
}

#[test]
fn when_file_passes_while_the_file_exists() {
    let gate = Custom::WhenFile(fixture("ready"));
    assert_eq!(gate.check(), Verdict::Pass);
}

#[test]
fn when_file_fails_while_the_file_is_missing() {
    let gate = Custom::WhenFile(fixture("missing"));
    assert!(fails(&gate.check()));
}

#[test]
fn renders_as_the_switch() {
    let when = Custom::When("make ready".to_owned());
    assert_eq!(when.to_string(), "--when make ready");
    let file = Custom::WhenFile(PathBuf::from("/tmp/go"));
    assert_eq!(file.to_string(), "--when-file /tmp/go");
}
