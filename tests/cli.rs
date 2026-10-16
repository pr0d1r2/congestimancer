//! `cgmr` as a user runs it: the binary, its argv and its exit code
//! (tests:V1, V2). Every run reads kernel files from a fixture tree through
//! `CGMR_FIXTURE_ROOT`, never from the live machine, and every gate chosen
//! here decides the same on any host.

use std::error::Error;
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use assert_cmd::cargo::CommandCargoExt;
use predicates::prelude::*;

type Outcome = Result<(), Box<dyn Error>>;

/// The fixture tree that stands for `/` (tests:V1).
const FIXTURE: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cpu/linux");
/// A file that exists wherever the tests run.
const HERE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
/// A file that exists nowhere: its `--when-file` gate always fails.
const NOWHERE: &str = "/no/such/file/for/cgmr";
/// A gate no machine supports: CPU throttling read on the host, not a
/// cgroup.
const UNSUPPORTED: [&str; 6] = [
    "--cgroup",
    "host",
    "--cpu-throttle",
    "50",
    "--interval",
    "10ms",
];

/// `cgmr` on the fixture tree, with no `CGMR_*` variable from the caller.
fn cgmr() -> Result<assert_cmd::Command, Box<dyn Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("cgmr")?;
    for (name, _) in std::env::vars_os() {
        if name.to_string_lossy().starts_with("CGMR_") {
            cmd.env_remove(name);
        }
    }
    cmd.env("CGMR_FIXTURE_ROOT", FIXTURE);
    Ok(cmd)
}

#[test]
fn the_command_exits_with_its_own_code() -> Outcome {
    cgmr()?.args(["--", "sh", "-c", "exit 7"]).assert().code(7);
    let gated = ["--when-file", HERE, "--", "sh", "-c", "exit 0"];
    cgmr()?.args(gated).assert().code(0);
    Ok(())
}

#[test]
fn run_without_a_command_exits_0_once_the_gates_pass() -> Outcome {
    cgmr()?.assert().code(0);
    cgmr()?.args(["run", "--when-file", HERE]).assert().code(0);
    Ok(())
}

#[test]
fn check_exits_0_on_pass_and_1_on_fail() -> Outcome {
    cgmr()?
        .args(["check", "--when-file", HERE])
        .assert()
        .code(0);
    cgmr()?
        .args(["check", "--when-file", NOWHERE])
        .assert()
        .code(1);
    Ok(())
}

#[test]
fn a_usage_error_exits_64() -> Outcome {
    cgmr()?.arg("--no-such-switch").assert().code(64);
    cgmr()?.arg("frobnicate").assert().code(64);
    cgmr()?
        .args(["check", "--load", "lots"])
        .assert()
        .code(64)
        .stderr(predicate::str::starts_with("cgmr: --load: "));
    let twice = ["check", "--load", "1", "--load", "2"];
    cgmr()?.args(twice).assert().code(64);
    cgmr()?
        .arg("check")
        .env("CGMR_DISK", "huge")
        .assert()
        .code(64);
    Ok(())
}

#[test]
fn an_unsupported_gate_exits_69() -> Outcome {
    cgmr()?
        .args(UNSUPPORTED)
        .assert()
        .code(69)
        .stderr(predicate::str::contains("--cpu-throttle"));
    cgmr()?.arg("check").args(UNSUPPORTED).assert().code(69);
    Ok(())
}

#[test]
fn the_timeout_exits_124_and_the_command_never_runs() -> Outcome {
    cgmr()?
        .args(["--when-file", NOWHERE, "--interval", "10ms"])
        .args(["--timeout", "50ms", "--", "sh", "-c", "exit 7"])
        .assert()
        .code(124)
        .stderr(predicate::str::contains("--timeout"));
    Ok(())
}

/// Starts a `cgmr` that waits forever, sends it `signal` once it waits,
/// and returns the signal it died of.
fn killed_while_waiting(signal: &str) -> Result<Option<i32>, Box<dyn Error>> {
    let mut child = Command::cargo_bin("cgmr")?
        .env("CGMR_FIXTURE_ROOT", FIXTURE)
        .args(["--when-file", NOWHERE, "--interval", "10ms"])
        .args(["--", "sh", "-c", "exit 7"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    sleep(Duration::from_millis(300));
    let pid = child.id().to_string();
    let sent = Command::new("kill").args([signal, &pid]).status()?;
    assert!(sent.success());
    Ok(child.wait()?.signal())
}

#[test]
fn sigint_and_sigterm_while_waiting_keep_the_default_disposition() -> Outcome {
    // A shell reports these deaths as 130 and 143 (.:V22).
    assert_eq!(killed_while_waiting("-INT")?, Some(2));
    assert_eq!(killed_while_waiting("-TERM")?, Some(15));
    Ok(())
}
