//! Each real `--govern` run happens in a child test process of its own
//! (`probe`): it forks, blocks signals and hands slot locks over, none of
//! which may touch the other tests' threads, signals and lock files.

use std::collections::VecDeque;
use std::ffi::OsString;
use std::io;
use std::os::unix::process::ExitStatusExt as _;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

use super::{Governing, Spawned, launch_governed};
use crate::gate::Overall;
use crate::launch::scratch::Scratch;
use crate::launch::{Group, Judge, Slots, Start};
use crate::slot::Release;
use crate::sys::GroupSignal;
use crate::wait::Admit;

const INTERVAL: Duration = Duration::from_millis(20);

/// Set only in the child process a test below starts: names the case
/// [`probe`] runs.
const PROBE: &str = "LAUNCH_GOVERN_PROBE";

/// Rounds that say `script` in order, then pass.
struct Script(VecDeque<Overall>);

impl Script {
    fn new(script: &[Overall]) -> Self {
        Self(script.iter().copied().collect())
    }
}

impl Judge for Script {
    fn judge(&mut self, _: Instant) -> Overall {
        self.0.pop_front().unwrap_or(Overall::Pass)
    }
}

fn sh(script: &str) -> [OsString; 3] {
    ["/bin/sh", "-c", script].map(OsString::from)
}

/// Governs `argv` by `judge`; its log goes to stderr, for the parent.
fn run(argv: &[OsString], start: Start, judge: &mut dyn Judge) -> u8 {
    let mut log = io::stderr();
    let governing = Governing::new(judge, INTERVAL, &mut log);
    launch_governed(argv, start, Vec::new(), governing)
}

/// Governs `argv` with no slot, its rounds saying `rounds`.
fn plain(argv: &[OsString], rounds: &[Overall]) -> u8 {
    run(argv, Start::default(), &mut Script::new(rounds))
}

/// Runs `case` in a child test process: its exit code and stderr.
fn in_child(case: &str) -> io::Result<(Option<i32>, String)> {
    let Output { status, stderr, .. } = Command::new(std::env::current_exe()?)
        .args(["--exact", "launch::governed::tests::probe", "--nocapture"])
        .env(PROBE, case)
        .output()?;
    Ok((status.code(), String::from_utf8_lossy(&stderr).into_owned()))
}

#[test]
fn probe() -> io::Result<()> {
    let Some(case) = std::env::var_os(PROBE) else {
        return Ok(());
    };
    let fail_twice = [Overall::Fail, Overall::Fail];
    let missing = [OsString::from("/nonexistent-cgmr-govern/cmd")];
    let code = match case.to_str() {
        Some("exit") => plain(&sh("sleep 0.2; exit 3"), &[]),
        Some("pause") => plain(&sh("sleep 0.2; exit 4"), &fail_twice),
        Some("missing") => plain(&missing, &[]),
        Some("term") => term()?,
        Some("slot") => slot()?,
        _ => 99,
    };
    std::process::exit(i32::from(code))
}

#[test]
fn a_governed_command_passes_its_exit_code_on() -> io::Result<()> {
    let (code, log) = in_child("exit")?;
    assert_eq!(code, Some(3), "{log}");
    Ok(())
}

#[test]
fn a_failing_round_pauses_the_command_and_a_pass_resumes_it() -> io::Result<()>
{
    let (code, log) = in_child("pause")?;
    assert_eq!(code, Some(4), "{log}");
    assert!(log.contains("paused"), "{log}");
    assert!(log.contains("resumed"), "{log}");
    Ok(())
}

#[test]
fn no_command_exits_0() {
    let mut log = Vec::new();
    let mut judge = Script::new(&[]);
    let governing = Governing::new(&mut judge, INTERVAL, &mut log);
    assert_eq!(
        launch_governed(&[], Start::default(), Vec::new(), governing),
        0
    );
}

#[test]
fn a_missing_command_exits_127() -> io::Result<()> {
    let (code, log) = in_child("missing")?;
    assert_eq!(code, Some(127), "{log}");
    Ok(())
}

/// 0 when a spawned group dies of a `SIGTERM` sent to it.
fn term() -> io::Result<u8> {
    let mut command = Command::new("sleep");
    command.arg("5");
    let mut spawned = Spawned::spawn(&mut command)?;
    spawned.signal(GroupSignal::Term)?;
    let end = Instant::now().checked_add(Duration::from_secs(2));
    while end.is_some_and(|end| Instant::now() < end) {
        if let Some(status) = spawned.try_wait()? {
            return Ok(u8::from(status.signal() != Some(libc::SIGTERM)));
        }
        spawned.wait(INTERVAL)?;
    }
    Ok(2)
}

#[test]
fn the_spawned_group_dies_of_a_forwarded_signal() -> io::Result<()> {
    let (code, log) = in_child("term")?;
    assert_eq!(code, Some(0), "{log}");
    Ok(())
}

/// A judge that, each round, sees whether another `cgmr` could take the
/// slot, then says `script`.
struct SlotWatch {
    script: Script,
    other: Slots,
    free: Vec<bool>,
}

impl Judge for SlotWatch {
    fn judge(&mut self, now: Instant) -> Overall {
        let admitted = self.other.try_admit();
        self.free.push(!matches!(admitted, Ok(None)));
        self.script.judge(now)
    }
}

fn claim(scratch: &Scratch) -> io::Result<Release> {
    let mut slots = Slots::new(scratch.waiter(Some(1 << 20))?);
    match slots.try_admit().map_err(io::Error::other)? {
        Some(Some(release)) => Ok(release),
        other => Err(io::Error::other(format!("not released: {other:?}"))),
    }
}

/// 0 when the slot stays taken through every round, paused or not, and
/// is free once the command has ended.
fn slot() -> io::Result<u8> {
    let scratch = Scratch::new("govern-slot")?;
    let start = Start::new(Some(claim(&scratch)?), false);
    let mut watch = SlotWatch {
        script: Script::new(&[Overall::Fail, Overall::Fail]),
        other: Slots::new(scratch.waiter(None)?),
        free: Vec::new(),
    };
    let code = run(&sh("sleep 0.2"), start, &mut watch);
    let held = !watch.free.is_empty() && !watch.free.contains(&true);
    let after = watch.other.try_admit().map_err(io::Error::other)?;
    let freed = matches!(after, Some(Some(_)));
    eprintln!("code {code}, held {held} {:?}, freed {freed}", watch.free);
    Ok(u8::from(!(code == 0 && held && freed)))
}

#[test]
fn a_paused_command_keeps_its_slot_until_it_ends() -> io::Result<()> {
    let (code, log) = in_child("slot")?;
    assert_eq!(code, Some(0), "{log}");
    assert!(log.contains("paused"), "{log}");
    Ok(())
}
