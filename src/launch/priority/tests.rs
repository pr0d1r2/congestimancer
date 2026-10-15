use std::io::{self, Write};
use std::process::Command;

use super::{BACKGROUND_NICE, background, report};
use crate::sys::{IoPriority, io_priority, nice};

/// Set only in the child [`background_lowers_cpu_and_io_priority`]
/// spawns, so lowering priority never slows the other tests.
const PROBE: &str = "LAUNCH_PRIORITY_PROBE";

#[test]
fn probe() -> io::Result<()> {
    if std::env::var_os(PROBE).is_none() {
        return Ok(());
    }
    background(&mut io::stderr());
    let io = io_priority()? == IoPriority::Background;
    let mut out = io::stdout();
    writeln!(out, "probe: nice={} background_io={io}", nice()?)
}

#[test]
fn background_lowers_cpu_and_io_priority() -> io::Result<()> {
    let output = Command::new(std::env::current_exe()?)
        .args(["--exact", "launch::priority::tests::probe", "--nocapture"])
        .env(PROBE, "1")
        .output()?;
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let want = format!("probe: nice={BACKGROUND_NICE} background_io=true");
    assert!(stdout.contains(&want), "{stdout}");
    Ok(())
}

#[test]
fn a_refused_call_is_reported_and_not_fatal() {
    let mut log = Vec::new();
    let denied = io::Error::from(io::ErrorKind::PermissionDenied);
    report(&mut log, "IO priority", Err(denied));
    let text = String::from_utf8_lossy(&log);
    assert!(
        text.starts_with("cgmr: cannot lower IO priority: "),
        "{text}"
    );
}

#[test]
fn success_says_nothing() {
    let mut log = Vec::new();
    report(&mut log, "nice", Ok(()));
    assert!(log.is_empty());
}
