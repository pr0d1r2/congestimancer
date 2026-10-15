use std::io;
use std::os::unix::process::{CommandExt as _, ExitStatusExt as _};
use std::process::{Child, Command};

use super::{GroupSignal, signal_group};

/// A `sleep` long enough to outlive the test, leader of its own group.
fn sleeper() -> io::Result<Child> {
    Command::new("sleep").arg("30").process_group(0).spawn()
}

#[test]
fn term_ends_every_process_in_the_group() -> io::Result<()> {
    let mut child = sleeper()?;
    signal_group(child.id(), GroupSignal::Term)?;
    assert_eq!(child.wait()?.signal(), Some(libc::SIGTERM));
    Ok(())
}

#[test]
fn a_stopped_group_continues_and_still_takes_a_signal() -> io::Result<()> {
    let mut child = sleeper()?;
    signal_group(child.id(), GroupSignal::Stop)?;
    signal_group(child.id(), GroupSignal::Cont)?;
    signal_group(child.id(), GroupSignal::Hup)?;
    assert_eq!(child.wait()?.signal(), Some(libc::SIGHUP));
    Ok(())
}

#[test]
fn int_reaches_the_group() -> io::Result<()> {
    let mut child = sleeper()?;
    signal_group(child.id(), GroupSignal::Int)?;
    assert_eq!(child.wait()?.signal(), Some(libc::SIGINT));
    Ok(())
}

#[test]
fn group_zero_is_refused_not_sent_to_ourselves() {
    let err = signal_group(0, GroupSignal::Term).err().map(|e| e.kind());
    assert_eq!(err, Some(io::ErrorKind::InvalidInput));
}

#[test]
fn a_group_past_pid_t_is_refused() {
    let err = signal_group(u32::MAX, GroupSignal::Cont).err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::InvalidInput));
}
