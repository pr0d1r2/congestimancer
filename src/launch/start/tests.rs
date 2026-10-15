use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io;
use std::os::fd::AsRawFd;
use std::process::Command;

use super::{Start, command, hand_over, launch};
use crate::launch::Slots;
use crate::launch::scratch::Scratch;
use crate::slot::{HELD_SLOTS_ENV, Release};
use crate::wait::Admit;

/// Whether a child started now sees `file`'s descriptor open.
fn child_sees(file: &File) -> io::Result<bool> {
    let fd = file.as_raw_fd();
    let test = format!("test -e /dev/fd/{fd}");
    Ok(Command::new("/bin/sh")
        .args(["-c", &test])
        .status()?
        .success())
}

fn claim(scratch: &Scratch) -> io::Result<Release> {
    let mut slots = Slots::new(scratch.waiter(Some(1 << 20))?);
    match slots.try_admit().map_err(io::Error::other)? {
        Some(Some(release)) => Ok(release),
        other => Err(io::Error::other(format!("not released: {other:?}"))),
    }
}

#[test]
fn the_command_inherits_the_slot_and_reservation() -> io::Result<()> {
    let scratch = Scratch::new("hand-over")?;
    let release = claim(&scratch)?;
    let (Some(slot), Some(reserved)) = (release.slot(), release.reservation())
    else {
        unreachable!("one slot and a reservation were asked: {release:?}");
    };
    assert!(!child_sees(slot.file())?, "close-on-exec before hand over");
    assert!(!child_sees(reserved.file())?, "close-on-exec before");
    hand_over(&release)?;
    assert!(child_sees(slot.file())?, "slot lock not inherited");
    assert!(child_sees(reserved.file())?, "reservation not inherited");
    Ok(())
}

/// Set only in the child [`exec_hands_over_the_exit_code`] spawns: that
/// child execs into a shell, which no other test may do.
const PROBE: &str = "LAUNCH_EXEC_PROBE";

#[test]
fn exec_probe() {
    if std::env::var_os(PROBE).is_none() {
        return;
    }
    let script = "test -z \"$CGMR_LOAD\" && exit 7; exit 1";
    let argv = ["/bin/sh", "-c", script].map(OsString::from);
    let vars = std::env::vars_os();
    let code = launch(&argv, Start::new(None, true), vars, &mut io::stderr());
    unreachable!("exec failed with {code}");
}

#[test]
fn exec_hands_over_the_exit_code() -> io::Result<()> {
    let status = Command::new(std::env::current_exe()?)
        .args(["--exact", "launch::start::tests::exec_probe"])
        .env(PROBE, "1")
        .env("CGMR_LOAD", "2")
        .status()?;
    assert_eq!(status.code(), Some(7));
    Ok(())
}

#[test]
fn no_command_exits_0() {
    let mut log = Vec::new();
    assert_eq!(launch(&[], Start::default(), Vec::new(), &mut log), 0);
    assert!(log.is_empty());
}

#[test]
fn a_missing_command_still_exits_127() {
    let argv = [OsString::from("/nonexistent-cgmr-launch/cmd")];
    let mut log = Vec::new();
    assert_eq!(launch(&argv, Start::default(), Vec::new(), &mut log), 127);
}

#[test]
fn the_command_gets_its_arguments_and_a_clean_environment() {
    let argv = ["make", "-j8"].map(OsString::from);
    let vars = [("CGMR_LOAD", "2")].map(|(k, v)| (k.into(), v.into()));
    let Some(built) = command(&argv, vars, Some("make")) else {
        unreachable!("a command was given");
    };
    assert_eq!(built.get_program(), "make");
    assert_eq!(built.get_args().collect::<Vec<_>>(), ["-j8"]);
    let envs: Vec<_> = built.get_envs().collect();
    let removed = (OsStr::new("CGMR_LOAD"), None);
    let held = (OsStr::new(HELD_SLOTS_ENV), Some(OsStr::new("make")));
    assert_eq!(envs, [held, removed]);
}
