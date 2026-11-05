//! `--govern-by cgroup` from the binary's side (src/launch:V12): a bad
//! mode is a usage error, and without a writable delegated cgroup the
//! run exits 69 before the command starts. Where the machine does
//! delegate one, the command runs inside it and its code comes back.

use super::{Outcome, cgmr};

/// `cgmr --run-timeout 30s --govern-by MODE -- sh -c SCRIPT`, with a lock
/// dir of its own.
fn by(
    mode: &str,
    script: &str,
) -> Result<std::process::Output, std::io::Error> {
    let state = tempfile::tempdir()?;
    let mut cmd = cgmr().map_err(|e| std::io::Error::other(e.to_string()))?;
    cmd.env("XDG_RUNTIME_DIR", state.path());
    cmd.env("TMPDIR", state.path());
    cmd.args(["--run-timeout", "30s", "--govern-by", mode]);
    cmd.args(["--", "sh", "-c", script]).output()
}

#[test]
fn an_unknown_mode_is_a_usage_error() -> Outcome {
    let out = by("pid", "exit 0")?;
    assert_eq!(out.status.code(), Some(64));
    Ok(())
}

#[test]
fn group_mode_runs_the_command_as_before() -> Outcome {
    let out = by("group", "exit 5")?;
    assert_eq!(out.status.code(), Some(5));
    Ok(())
}

#[test]
fn cgroup_mode_runs_it_or_refuses_with_69_and_says_why() -> Outcome {
    let out = by("cgroup", "exit 5")?;
    let err = String::from_utf8_lossy(&out.stderr);
    match out.status.code() {
        Some(5) => {}
        Some(69) => assert!(err.contains("cgroup"), "{err}"),
        other => return Err(format!("exit {other:?}: {err}").into()),
    }
    Ok(())
}
