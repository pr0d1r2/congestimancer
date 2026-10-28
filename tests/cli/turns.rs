//! `--govern` jobs that share a `--slot-key` take turns (src/launch:V10):
//! when the gate fails the youngest stops alone, and when it passes again
//! it resumes. The gate is a file the test removes and brings back.

use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use assert_cmd::cargo::CommandCargoExt;

use super::{FIXTURE, Outcome};

/// `cgmr --govern --slot-key turns` gated on `gate`, over a `sleep` whose
/// process id lands in `pidfile`.
fn spawn(
    runtime: &Path,
    gate: &Path,
    pidfile: &Path,
) -> Result<Child, Box<dyn Error>> {
    let script = format!("echo $$ > '{}'; exec sleep 60", pidfile.display());
    let child = Command::cargo_bin("cgmr")?
        .env("CGMR_FIXTURE_ROOT", FIXTURE)
        .env("XDG_RUNTIME_DIR", runtime)
        .args(["--govern", "--slot-key", "turns", "--interval", "50ms"])
        .args(["--settle", "100ms", "--when-file"])
        .arg(gate)
        .args(["--", "sh", "-c", &script])
        .stdin(Stdio::null())
        .spawn()?;
    Ok(child)
}

/// A governed job: `cgmr` and the process id of the `sleep` it governs.
struct Job {
    /// The `cgmr` process.
    cgmr: Child,
    /// The governed command.
    pid: u32,
}

impl Job {
    /// Starts a governed `sleep` of key `turns`, gated on `gate`, whose
    /// process id lands in `pidfile`.
    fn start(
        runtime: &Path,
        gate: &Path,
        pidfile: &Path,
    ) -> Result<Self, Box<dyn Error>> {
        let cgmr = spawn(runtime, gate, pidfile)?;
        let pid = (0..500).find_map(|_| {
            sleep(Duration::from_millis(10));
            fs::read_to_string(pidfile).ok()?.trim().parse().ok()
        });
        let pid = pid.ok_or("the job never started")?;
        Ok(Self { cgmr, pid })
    }

    /// Whether the command is stopped (`T` in `/proc/<pid>/stat`).
    fn stopped(&self) -> bool {
        let stat = fs::read_to_string(format!("/proc/{}/stat", self.pid));
        let state = stat.ok().and_then(|s| {
            let rest = s.rsplit_once(')')?.1.trim_start().to_owned();
            rest.chars().next()
        });
        state == Some('T')
    }

    /// Ends `cgmr`, which passes `SIGTERM` on to its command.
    fn end(mut self) {
        let _ = Command::new("kill")
            .arg(self.cgmr.id().to_string())
            .status();
        let _ = self.cgmr.wait();
    }
}

/// Whether `ok` holds within five seconds.
fn soon(ok: impl Fn() -> bool) -> bool {
    (0..500).any(|_| {
        sleep(Duration::from_millis(10));
        ok()
    })
}

#[test]
fn the_youngest_job_stops_alone_and_resumes_when_the_gate_passes() -> Outcome {
    let dir = tempfile::tempdir()?;
    let gate = dir.path().join("gate");
    fs::write(&gate, "")?;
    let old = Job::start(dir.path(), &gate, &dir.path().join("old"))?;
    let young = Job::start(dir.path(), &gate, &dir.path().join("young"))?;
    fs::remove_file(&gate)?;
    let paused = soon(|| young.stopped());
    // Many intervals on, the older job is still the one running.
    sleep(Duration::from_millis(500));
    let kept = !old.stopped();
    fs::write(&gate, "")?;
    let resumed = soon(|| !young.stopped());
    young.end();
    old.end();
    assert!(paused && kept && resumed, "{paused} {kept} {resumed}");
    Ok(())
}
