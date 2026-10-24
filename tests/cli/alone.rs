//! `--alone-passes` from the binary's side (src/slot:V12): the first job
//! of a slot key starts past gates that never pass, and a job that comes
//! while the key is held waits on them. Each test has a lock dir of its
//! own under `XDG_RUNTIME_DIR`.

use std::error::Error;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use assert_cmd::cargo::CommandCargoExt;

use super::{FIXTURE, NOWHERE, Outcome, cgmr};

/// The slot switches of an `--alone-passes` job behind a gate that never
/// passes.
const ALONE: [&str; 11] = [
    "--alone-passes",
    "--slots",
    "2",
    "--slot-key",
    "alone",
    "--when-file",
    NOWHERE,
    "--interval",
    "10ms",
    "--timeout",
    "300ms",
];

/// A job that exits 7 at once.
const JOB: [&str; 4] = ["--", "sh", "-c", "exit 7"];

/// `cgmr` on the fixture tree with its lock dir under `runtime`.
fn cgmr_in(runtime: &Path) -> Result<assert_cmd::Command, Box<dyn Error>> {
    let mut cmd = cgmr()?;
    cmd.env("XDG_RUNTIME_DIR", runtime);
    Ok(cmd)
}

#[test]
fn alone_passes_starts_the_first_job_of_a_key_past_failing_gates() -> Outcome {
    let runtime = tempfile::tempdir()?;
    let alone = cgmr_in(runtime.path())?.args(ALONE).args(JOB).assert();
    alone.code(7);
    let gated = cgmr_in(runtime.path())?
        .args(&ALONE[1..])
        .args(JOB)
        .assert();
    gated.code(124);
    Ok(())
}

/// Starts an `--alone-passes` job under `runtime` that holds its slot
/// until killed, and returns it once it runs.
fn holding_the_key(runtime: &Path) -> Result<Child, Box<dyn Error>> {
    let started = runtime.join("started");
    let script = format!("touch '{}' && exec sleep 30", started.display());
    let child = Command::cargo_bin("cgmr")?
        .env("CGMR_FIXTURE_ROOT", FIXTURE)
        .env("XDG_RUNTIME_DIR", runtime)
        .args(ALONE)
        .args(["--", "sh", "-c", &script])
        .stdin(Stdio::null())
        .spawn()?;
    if appears(&started) {
        Ok(child)
    } else {
        Err("the first job never started".into())
    }
}

/// Whether `path` exists within five seconds.
fn appears(path: &Path) -> bool {
    (0..500).any(|_| {
        sleep(Duration::from_millis(10));
        path.exists()
    })
}

#[test]
fn alone_passes_leaves_later_jobs_of_a_key_to_the_gates() -> Outcome {
    let runtime = tempfile::tempdir()?;
    let mut first = holding_the_key(runtime.path())?;
    let second = cgmr_in(runtime.path())?.args(ALONE).args(JOB).assert();
    first.kill()?;
    first.wait()?;
    second.code(124);
    Ok(())
}
