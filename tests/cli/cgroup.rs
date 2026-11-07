//! `--govern-by cgroup` from the binary's side (src/launch:V12): a bad
//! mode is a usage error, and without a writable delegated cgroup the
//! run exits 69 before the command starts. Where the machine does
//! delegate one, the command runs inside its `cgmr-<pid>` and its code
//! comes back.

use super::{NOWHERE, Outcome, cgmr};

/// `cgmr SWITCHES -- sh -c SCRIPT`, with a lock dir of its own.
fn with(
    switches: &[&str],
    script: &str,
) -> Result<std::process::Output, std::io::Error> {
    let state = tempfile::tempdir()?;
    let mut cmd = cgmr().map_err(|e| std::io::Error::other(e.to_string()))?;
    cmd.env("XDG_RUNTIME_DIR", state.path());
    cmd.env("TMPDIR", state.path());
    cmd.args(switches);
    cmd.args(["--", "sh", "-c", script]).output()
}

/// `cgmr --run-timeout 30s --govern-by MODE -- sh -c SCRIPT`.
fn by(
    mode: &str,
    script: &str,
) -> Result<std::process::Output, std::io::Error> {
    with(&["--run-timeout", "30s", "--govern-by", mode], script)
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
fn cgroup_mode_runs_it_where_delegated_else_refuses_with_69() -> Outcome {
    // 5 only from inside a `cgmr-<pid>`, not from the cgroup cgmr is in.
    let inside = "grep -q '/cgmr-[0-9]*$' /proc/self/cgroup && exit 5; exit 6";
    let out = by("cgroup", inside)?;
    let err = String::from_utf8_lossy(&out.stderr);
    // The binary runs in this test's cgroup, so the library's probe says
    // which outcome is due: a refusal where one is delegated fails. It is
    // sampled after the run, so it is taken no earlier than the binary's
    // own check.
    let delegated = congestimancer::delegated_cgroup().is_ok();
    let want = if delegated { 5 } else { 69 };
    assert_eq!(out.status.code(), Some(want), "{err}");
    assert!(delegated || err.contains("cgroup"), "{err}");
    Ok(())
}

#[test]
fn cgroup_mode_without_govern_or_run_timeout_execs_as_before() -> Outcome {
    // Nothing is governed, so no cgroup is wanted: a `CGMR_GOVERN_BY`
    // or profile line set for governed runs leaves plain runs alone.
    let out = with(&["--govern-by", "cgroup"], "exit 5")?;
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(5), "{err}");
    Ok(())
}

#[test]
fn a_missing_delegated_cgroup_is_refused_before_the_wait() -> Outcome {
    // Gates that never pass end the wait with 124; a cgroup that cannot
    // be had must be refused first, with 69, not after the wait.
    let delegated = congestimancer::delegated_cgroup().is_ok();
    let gated = ["--govern", "--govern-by", "cgroup", "--timeout", "1s"];
    let never = ["--when-file", NOWHERE, "--interval", "10ms"];
    let out = with(&[&gated[..], &never[..]].concat(), "exit 5")?;
    let err = String::from_utf8_lossy(&out.stderr);
    let want = if delegated { 124 } else { 69 };
    assert_eq!(out.status.code(), Some(want), "{err}");
    Ok(())
}
