//! `--run-timeout` from the binary's side (src/launch:V11): the command
//! is killed with its whole group once it has run the budget, and cgmr
//! leaves with the overrun code. Each test has a lock dir of its own.

use super::{Outcome, cgmr};

/// A gate-free `--run-timeout` run of `script` under `sh`.
fn timed(
    budget: &str,
    extra: &[&str],
    script: &str,
) -> assert_cmd::assert::Assert {
    let state = tempfile::tempdir().map(|d| d.keep());
    let state = state.unwrap_or_default();
    let mut cmd = cgmr().unwrap_or_else(|e| unreachable!("{e}"));
    cmd.env("XDG_RUNTIME_DIR", &state).env("TMPDIR", &state);
    cmd.args(["--run-timeout", budget]).args(extra);
    cmd.args(["--", "sh", "-c", script]).assert()
}

#[test]
fn an_overrunning_command_is_killed_and_exits_125() -> Outcome {
    timed("200ms", &[], "sleep 30").code(125);
    Ok(())
}

#[test]
fn the_overrun_code_is_configurable() -> Outcome {
    timed("200ms", &["--run-timeout-code", "99"], "sleep 30").code(99);
    Ok(())
}

#[test]
fn a_command_within_its_budget_keeps_its_own_code() -> Outcome {
    timed("30s", &[], "exit 7").code(7);
    Ok(())
}

#[test]
fn a_code_above_255_is_a_usage_error() -> Outcome {
    timed("1s", &["--run-timeout-code", "256"], "true").code(64);
    Ok(())
}
