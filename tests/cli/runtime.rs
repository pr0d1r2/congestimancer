//! `--run-timeout` from the binary's side (src/launch:V11): the command
//! is killed with its whole group once it has run the budget, and cgmr
//! leaves with the overrun code. Each test has a lock dir of its own.

use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use assert_cmd::cargo::CommandCargoExt;

use super::{FIXTURE, Outcome, cgmr};

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

/// `cgmr --run-timeout` over a script that leaves a `sleep` behind, with
/// its stdout piped back and the descendant's pid written to `pid_file`.
fn spawn_overrun(pid_file: &Path) -> Result<Child, Box<dyn Error>> {
    let script = "sleep 30 & echo $! > \"$DESCENDANT_PID_FILE\"; wait";
    let state = pid_file.parent().ok_or("pid file has no dir")?;
    let mut cmd = Command::cargo_bin("cgmr")?;
    for (name, _) in std::env::vars_os() {
        if name.to_string_lossy().starts_with("CGMR_") {
            cmd.env_remove(name);
        }
    }
    cmd.env("CGMR_FIXTURE_ROOT", FIXTURE);
    cmd.env("XDG_RUNTIME_DIR", state).env("TMPDIR", state);
    cmd.env("DESCENDANT_PID_FILE", pid_file);
    cmd.args(["--run-timeout", "500ms", "--", "sh", "-c", script]);
    Ok(cmd.stdout(Stdio::piped()).spawn()?)
}

/// Reads `stdout` to EOF off-thread; the channel gets `true` at EOF.
fn eof_of(mut stdout: ChildStdout) -> mpsc::Receiver<bool> {
    let (send, receive) = mpsc::channel();
    thread::spawn(move || {
        let mut sink = Vec::new();
        let _ = send.send(stdout.read_to_end(&mut sink).is_ok());
    });
    receive
}

/// The whole group dies with the command: a descendant holding the
/// stdout pipe closes it only by exiting, so EOF proves it is gone (a
/// `kill -0` probe would also succeed on a zombie).
#[test]
fn the_overrun_kills_the_whole_group() -> Outcome {
    let state = tempfile::tempdir()?;
    let pid_file = state.path().join("pid");
    let mut child = spawn_overrun(&pid_file)?;
    let stdout = child.stdout.take().ok_or("missing stdout pipe")?;
    let receive = eof_of(stdout);
    assert_eq!(child.wait()?.code(), Some(125));
    let pid = fs::read_to_string(&pid_file)?.trim().to_owned();
    let closed = receive.recv_timeout(Duration::from_secs(5));
    if closed.is_err() {
        let _ = Command::new("kill").args(["-KILL", &pid]).status();
    }
    assert!(closed.unwrap_or(false), "descendant {pid} kept the pipe");
    Ok(())
}

/// The wait for the gates is not run time: a gate that opens only after
/// twice the budget still leaves the command its whole budget.
#[test]
fn the_wait_before_the_launch_does_not_count() -> Outcome {
    let state = tempfile::tempdir()?;
    let gate = state.path().join("open");
    let path = gate.to_str().ok_or("temp dir is not UTF-8")?.to_owned();
    let opener = thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        fs::write(&gate, "")
    });
    let wait = ["--when-file", &path, "--interval", "50ms"];
    timed("1s", &wait, "sleep 0.3; exit 7").code(7);
    opener.join().map_err(|_| "the gate opener panicked")??;
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
