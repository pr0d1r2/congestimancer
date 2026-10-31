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

use super::{GroupAction, govern_step, status_code};
use crate::gate::{Gates, Overall, evaluate};
use crate::signal::{Clock, Source, Trends};
use crate::sys::{ForwardSignal, GroupSignal, Wake};

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
        evaluate(self.gates, self.source, now, &mut self.trends).overall
    }
}

/// The other `--govern` jobs of one slot key, which this job takes turns
/// with (src/launch:V10).
pub trait Mates {
    /// One round alongside the crew: works out what `overall` and the
    /// `calm` time the gates have passed without a break call for, and
    /// calls `act` with that action; the crew records it only when `act`
    /// says it was done.
    ///
    /// # Errors
    ///
    /// When the crew cannot be read or written.
    fn round(
        &mut self,
        overall: Overall,
        calm: Duration,
        act: &mut dyn FnMut(GroupAction) -> bool,
    ) -> io::Result<()>;

    /// This job runs again without a round (a forwarded signal).
    fn resumed(&mut self);
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
    /// The other governed jobs of the slot key, once there is a crew.
    pub mates: Option<&'a mut dyn Mates>,
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
            mates: None,
        }
    }

    /// This rig, taking turns with the crew `mates`.
    #[must_use]
    pub fn with_mates(mut self, mates: &'a mut dyn Mates) -> Self {
        self.mates = Some(mates);
        self
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
    let mut run = Run {
        rig,
        interval,
        stopped: false,
        last: None,
        calm: None,
    };
    loop {
        if let Some(code) = run.step()? {
            return Ok(code);
        }
    }
}

/// One `--govern` run's state between rounds.
struct Run<'a, J: ?Sized, C: ?Sized, G: ?Sized> {
    /// Its parts.
    rig: GovernRig<'a, J, C, G>,
    /// How often a round judges the gates.
    interval: Duration,
    /// Whether the group is stopped now.
    stopped: bool,
    /// When the last round ran, if one has.
    last: Option<Instant>,
    /// Since when the gates have passed without a break, if they do.
    calm: Option<Instant>,
}

impl<'a, J, C, G> Run<'a, J, C, G>
where
    J: Judge + ?Sized,
    C: Clock + ?Sized,
    G: Group + ?Sized,
{
    /// Waits for the next round, signal or exit, and acts on it; the
    /// exit code once the command has ended.
    fn step(&mut self) -> io::Result<Option<u8>> {
        if let Some(status) = self.rig.group.try_wait()? {
            self.release();
            return Ok(Some(status_code(status)));
        }
        match self.rig.group.wait(self.left())? {
            None => self.round(),
            Some(Wake::Forward(sig)) => self.forward(sig),
            Some(Wake::Child) => {}
        }
        Ok(None)
    }

    /// How long until the next round is due: a whole interval after the
    /// last one (the first comes an interval after the start, whose
    /// gates just passed).
    fn left(&self) -> Duration {
        let since = self.last.map_or(Duration::ZERO, |last| {
            self.rig.clock.now().saturating_duration_since(last)
        });
        self.interval.saturating_sub(since)
    }

    /// Judges the gates and stops or continues the group as they say, or
    /// as the crew's turn allows.
    fn round(&mut self) {
        let now = self.rig.clock.now();
        self.last = Some(now);
        let overall = self.rig.judge.judge(now);
        self.calm = match overall {
            Overall::Pass => self.calm.or(Some(now)),
            _ => None,
        };
        match self.rig.mates.take() {
            Some(mates) => self.crewed(mates, overall, now),
            None => self.alone(overall),
        }
    }

    /// A round whose action `mates` decides.
    fn crewed(
        &mut self,
        mates: &'a mut dyn Mates,
        overall: Overall,
        now: Instant,
    ) {
        let since = self.calm.map(|calm| now.saturating_duration_since(calm));
        let mut act = |action| self.act(action);
        let calm = since.unwrap_or(Duration::ZERO);
        if let Err(err) = mates.round(overall, calm, &mut act) {
            let _ = writeln!(
                self.rig.log,
                "cgmr: cannot judge the crew, governing alone: {err}"
            );
            return;
        }
        self.rig.mates = Some(mates);
    }

    /// Stops or continues the group as `overall` says, on its own.
    fn alone(&mut self, overall: Overall) {
        if let Some(action) = govern_step(self.stopped, overall) {
            self.act(action);
        }
    }

    /// Does `action`; whether the group is in the state it asks for.
    fn act(&mut self, action: GroupAction) -> bool {
        let (sig, stopped) = match action {
            GroupAction::Stop => (GroupSignal::Stop, true),
            GroupAction::Continue => (GroupSignal::Cont, false),
        };
        self.send(sig);
        self.stopped == stopped
    }

    /// Passes `sig` on; a stopped group is continued too, or it could not
    /// act on it.
    fn forward(&mut self, sig: ForwardSignal) {
        self.send(sig.group());
        self.release();
    }

    /// Continues the group if it is stopped.
    fn release(&mut self) {
        if self.stopped {
            self.send(GroupSignal::Cont);
            if let Some(mates) = self.rig.mates.as_deref_mut() {
                mates.resumed();
            }
        }
    }

    /// Sends `sig`, keeping track of whether the group is stopped; a
    /// refusal is logged and changes nothing.
    fn send(&mut self, sig: GroupSignal) {
        if let Err(err) = self.rig.group.signal(sig) {
            // The exit code tells the end; a lost stderr changes nothing.
            let _ = writeln!(self.rig.log, "cgmr: cannot signal: {err}");
            return;
        }
        let note = match sig {
            GroupSignal::Stop => Some((true, "gates fail, command paused")),
            GroupSignal::Cont => Some((false, "command resumed")),
            _ => None,
        };
        if let Some((stopped, note)) = note {
            self.stopped = stopped;
            let _ = writeln!(self.rig.log, "cgmr: {note}");
        }
    }
}

#[cfg(test)]
mod tests;
