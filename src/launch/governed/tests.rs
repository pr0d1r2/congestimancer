use std::collections::VecDeque;
use std::ffi::OsString;
use std::io;
use std::os::unix::process::ExitStatusExt as _;
use std::process::Command;
use std::time::{Duration, Instant};

use super::{Governing, Spawned, launch_governed};
use crate::gate::Overall;
use crate::launch::scratch::Scratch;
use crate::launch::{Group, Judge, Slots, Start};
use crate::slot::Release;
use crate::sys::GroupSignal;
use crate::wait::Admit;

const INTERVAL: Duration = Duration::from_millis(20);

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

fn run(argv: &[OsString], start: Start, judge: &mut dyn Judge) -> (u8, String) {
    let mut log = Vec::new();
    let governing = Governing::new(judge, INTERVAL, &mut log);
    let code = launch_governed(argv, start, Vec::new(), governing);
    (code, String::from_utf8_lossy(&log).into_owned())
}

#[test]
fn a_governed_command_passes_its_exit_code_on() {
    let argv = sh("sleep 0.2; exit 3");
    let (code, log) = run(&argv, Start::default(), &mut Script::new(&[]));
    assert_eq!(code, 3, "{log}");
}

#[test]
fn a_failing_round_pauses_the_command_and_a_pass_resumes_it() {
    let argv = sh("sleep 0.2; exit 4");
    let rounds = [Overall::Fail, Overall::Fail];
    let (code, log) = run(&argv, Start::default(), &mut Script::new(&rounds));
    assert_eq!(code, 4, "{log}");
    assert!(log.contains("paused"), "{log}");
    assert!(log.contains("resumed"), "{log}");
}

#[test]
fn no_command_exits_0() {
    assert_eq!(run(&[], Start::default(), &mut Script::new(&[])).0, 0);
}

#[test]
fn a_missing_command_exits_127() {
    let argv = [OsString::from("/nonexistent-cgmr-govern/cmd")];
    let (code, log) = run(&argv, Start::default(), &mut Script::new(&[]));
    assert_eq!(code, 127, "{log}");
}

#[test]
fn the_spawned_group_dies_of_a_forwarded_signal() -> io::Result<()> {
    let mut command = Command::new("sleep");
    command.arg("5");
    let mut spawned = Spawned::spawn(&mut command)?;
    spawned.signal(GroupSignal::Term)?;
    let end = Instant::now() + Duration::from_secs(2);
    while Instant::now() < end {
        if let Some(status) = spawned.try_wait()? {
            assert_eq!(status.signal(), Some(libc::SIGTERM));
            return Ok(());
        }
        spawned.wait(INTERVAL)?;
    }
    Err(io::Error::other("the command outlived SIGTERM"))
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

#[test]
fn a_paused_command_keeps_its_slot_until_it_ends() -> io::Result<()> {
    let scratch = Scratch::new("govern-slot")?;
    let start = Start::new(Some(claim(&scratch)?), false);
    let mut watch = SlotWatch {
        script: Script::new(&[Overall::Fail, Overall::Fail]),
        other: Slots::new(scratch.waiter(None)?),
        free: Vec::new(),
    };
    let (code, log) = run(&sh("sleep 0.2"), start, &mut watch);
    assert_eq!(code, 0, "{log}");
    assert!(log.contains("paused"), "{log}");
    assert!(!watch.free.is_empty() && !watch.free.contains(&true));
    let after = watch.other.try_admit().map_err(io::Error::other)?;
    assert!(matches!(after, Some(Some(_))), "{after:?}");
    Ok(())
}
