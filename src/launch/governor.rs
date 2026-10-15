//! [`govern`]: the `--govern` loop. The command runs in its own process
//! group; each interval the gates are judged again, a failing round stops
//! the group and a passing one continues it; `SIGINT`, `SIGTERM` and
//! `SIGHUP` pass on to the group; `cgmr` ends with the command's own code
//! (src/launch:V4).
//!
//! The loop is generic over its parts, so tests drive it with a
//! `ManualClock`, a `FakeSource` and a fake group:
//!
//! - [`Judge`]: what a round says; [`LocalGates`] for the gates on this
//!   machine. A caller with `--host` judges the remote gates too, each
//!   round (src/launch:V8).
//! - [`Group`]: the command's group: signal it, see whether the command
//!   ended, wait for a signal to forward.

use std::io::{self, Write};
use std::process::ExitStatus;
use std::time::{Duration, Instant};

use crate::gate::{Gates, Overall};
use crate::signal::{Clock, Source, Trends};
use crate::sys::{GroupSignal, Wake};

/// What one `--govern` round says about the gates.
pub trait Judge {
    /// The gates together at monotonic instant `now`.
    fn judge(&mut self, now: Instant) -> Overall;
}

/// The gates on this machine as a [`Judge`].
pub struct LocalGates<'a, S: ?Sized> {
    /// The gates to judge.
    gates: &'a mut Gates,
    /// Where their shared signals come from.
    source: &'a mut S,
    /// Kept across rounds, as the wait keeps them.
    trends: Trends,
}

impl<'a, S: ?Sized> LocalGates<'a, S> {
    /// `gates` read from `source`.
    pub fn new(gates: &'a mut Gates, source: &'a mut S) -> Self {
        Self {
            gates,
            source,
            trends: Trends::new(),
        }
    }
}

impl<S: Source + ?Sized> Judge for LocalGates<'_, S> {
    fn judge(&mut self, now: Instant) -> Overall {
        let _ = (now, &self.gates, &self.source, &self.trends);
        Overall::Unsupported
    }
}

/// The command's process group, as `--govern` drives it.
pub trait Group {
    /// Sends `sig` to every process in the group.
    ///
    /// # Errors
    ///
    /// When the signal cannot be sent.
    fn signal(&mut self, sig: GroupSignal) -> io::Result<()>;

    /// The command's exit status once it has ended, else `None`.
    ///
    /// # Errors
    ///
    /// When its state cannot be read.
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>>;

    /// Waits up to `d` for a signal to forward or a child to change
    /// state; `None` when `d` ran out.
    ///
    /// # Errors
    ///
    /// When the wait fails.
    fn wait(&mut self, d: Duration) -> io::Result<Option<Wake>>;
}

/// What one `--govern` run judges, reads the time from, drives and
/// writes to.
#[non_exhaustive]
pub struct GovernRig<'a, J: ?Sized, C: ?Sized, G: ?Sized> {
    /// What each round says.
    pub judge: &'a mut J,
    /// The monotonic clock the interval is measured on.
    pub clock: &'a C,
    /// The command's group.
    pub group: &'a mut G,
    /// Stderr: a line when the command pauses or resumes.
    pub log: &'a mut dyn Write,
}

impl<'a, J: ?Sized, C: ?Sized, G: ?Sized> GovernRig<'a, J, C, G> {
    /// A rig over these parts.
    pub fn new(
        judge: &'a mut J,
        clock: &'a C,
        group: &'a mut G,
        log: &'a mut dyn Write,
    ) -> Self {
        Self {
            judge,
            clock,
            group,
            log,
        }
    }
}

/// Governs the rig's group until its command ends, judging the gates
/// every `interval`; returns the exit code to leave with: the command's
/// own, or 128 + n for death by signal n (src/launch:V4).
///
/// # Errors
///
/// When the command's state cannot be read or the wait fails.
pub fn govern<J, C, G>(
    rig: GovernRig<'_, J, C, G>,
    interval: Duration,
) -> io::Result<u8>
where
    J: Judge + ?Sized,
    C: Clock + ?Sized,
    G: Group + ?Sized,
{
    let _ = (rig, interval);
    Err(io::Error::from(io::ErrorKind::Unsupported))
}

#[cfg(test)]
mod tests;
