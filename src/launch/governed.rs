//! [`launch_governed`]: `--govern`'s start. The command is spawned, not
//! exec'd, as the leader of its own process group, with `SIGINT`,
//! `SIGTERM` and `SIGHUP` blocked in `cgmr` only from then on, so a
//! signal while waiting still has its default disposition (`.:V22`).
//! `cgmr` holds the slot and reservation until the command ends; the
//! command inherits them too, so a paused command keeps them
//! (src/launch:V4, V8).

use std::ffi::OsString;
use std::io::{self, Write};
use std::process::{Child, Command, ExitStatus};
use std::time::Duration;

use super::{Group, Judge, Start};
use crate::sys::{Blocked, GroupSignal, Wake};

/// What `--govern` judges each round, how often, and where it writes.
#[non_exhaustive]
pub struct Governing<'a, J: ?Sized> {
    /// What each round says: [`LocalGates`](super::LocalGates), or the
    /// remote gates with `--host`.
    pub judge: &'a mut J,
    /// How often a round runs (`--interval`).
    pub interval: Duration,
    /// Stderr.
    pub log: &'a mut dyn Write,
}

impl<'a, J: ?Sized> Governing<'a, J> {
    /// Governing by `judge` every `interval`, writing to `log`.
    pub fn new(
        judge: &'a mut J,
        interval: Duration,
        log: &'a mut dyn Write,
    ) -> Self {
        Self {
            judge,
            interval,
            log,
        }
    }
}

/// Starts `argv` as `start` says (environment cleaned from `vars`, slot
/// handed over, background priority) and governs it until it ends;
/// returns the exit code to leave with: 0 with no command, 126 or 127
/// when it cannot start, else the command's own (src/launch:V4).
pub fn launch_governed<I, J>(
    argv: &[OsString],
    start: Start,
    vars: I,
    governing: Governing<'_, J>,
) -> u8
where
    I: IntoIterator<Item = (OsString, OsString)>,
    J: Judge + ?Sized,
{
    let _ = (argv, start, vars, governing, Spawned::spawn);
    1
}

/// The spawned command, with this thread's forwarded signals blocked.
#[derive(Debug)]
pub(super) struct Spawned {
    /// The command, leader of its own group.
    child: Child,
    /// Forwarded signals, blocked until drop.
    blocked: Blocked,
}

impl Spawned {
    /// Blocks the forwarded signals, then spawns `command` in a group of
    /// its own.
    pub(super) fn spawn(command: &mut Command) -> io::Result<Self> {
        let _ = command;
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}

impl Group for Spawned {
    fn signal(&mut self, sig: GroupSignal) -> io::Result<()> {
        let _ = (sig, &self.child);
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    fn wait(&mut self, d: Duration) -> io::Result<Option<Wake>> {
        let _ = (d, &self.blocked);
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}

#[cfg(test)]
mod tests;
