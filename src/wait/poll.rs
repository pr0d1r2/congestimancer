//! [`poll`]: evaluate the gates, sleep, again, until they release, the
//! timeout runs out, or a gate is unsupported (src/wait:T1).

use std::convert::Infallible;
use std::io::Write;
use std::time::{Duration, Instant};

use super::Pacing;
use crate::gate::{Evaluation, Gates, Overall, evaluate};
use crate::signal::{Clock, Source, Trends};

/// The slot steps that run once the gates pass and settle, before
/// `--jitter` (src/wait:V9); `src/slot`'s `Waiter` is the real one.
pub trait Admit {
    /// What the command holds while it runs (slot, reservation).
    type Held;
    /// Why a step could not be tried at all.
    type Error;

    /// `Some` when every step lets the command go, `None` when one says
    /// wait, which sends the loop back to the gates.
    ///
    /// # Errors
    ///
    /// When a step cannot be tried (the lock dir cannot be read).
    fn try_admit(&mut self) -> Result<Option<Self::Held>, Self::Error>;
}

/// No slot steps: the gates alone decide.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Open;

impl Admit for Open {
    type Held = ();
    type Error = Infallible;

    fn try_admit(&mut self) -> Result<Option<()>, Infallible> {
        Ok(Some(()))
    }
}

/// How a wait ended (src/wait interfaces).
#[derive(Debug)]
#[non_exhaustive]
pub enum Outcome<H> {
    /// The gates passed, settled, and every slot step let it go.
    Released(H),
    /// `--timeout` ran out; the last round's evaluation, if any.
    TimedOut(Option<Evaluation>),
    /// A gate has no source here (src/gate:V3).
    Unsupported(Evaluation),
}

impl<H> Outcome<H> {
    /// The exit code this outcome ends `cgmr` with, or `None` when the
    /// command runs: 124 for a timeout (src/wait:V3), 69 for an
    /// unsupported gate (src:V5).
    #[must_use]
    pub const fn exit_code(&self) -> Option<u8> {
        match self {
            Self::Released(_) => None,
            Self::TimedOut(_) => Some(124),
            Self::Unsupported(_) => Some(69),
        }
    }
}

/// What one wait reads, sleeps on, and writes to.
#[non_exhaustive]
pub struct Rig<'a, S: ?Sized, C: ?Sized> {
    /// The gates to pass.
    pub gates: &'a mut Gates,
    /// Where the shared signals come from.
    pub source: &'a mut S,
    /// The monotonic clock every duration is measured on (src/wait:V7).
    pub clock: &'a mut C,
    /// Stderr: heartbeat and verbose lines (src/report:V2).
    pub log: &'a mut dyn Write,
}

impl<'a, S: ?Sized, C: ?Sized> Rig<'a, S, C> {
    /// A rig over these parts.
    pub fn new(
        gates: &'a mut Gates,
        source: &'a mut S,
        clock: &'a mut C,
        log: &'a mut dyn Write,
    ) -> Self {
        Self {
            gates,
            source,
            clock,
            log,
        }
    }
}

/// Waits until the gates release the command, as `pacing` says.
///
/// Each round evaluates every gate at one [`Clock`] instant; between
/// rounds the clock sleeps, never spins (src/wait:V1, V7).
///
/// # Errors
///
/// When `admit` cannot try its steps.
pub fn poll<S, C, A>(
    mut rig: Rig<'_, S, C>,
    pacing: &Pacing,
    admit: &mut A,
) -> Result<Outcome<A::Held>, A::Error>
where
    S: Source + ?Sized,
    C: Clock + ?Sized,
    A: Admit,
{
    let mut run = Run::new(pacing, rig.clock.now());
    loop {
        let now = rig.clock.now();
        if let Some(outcome) = run.round(&mut rig, now, admit)? {
            return Ok(outcome);
        }
        if run.expired(now) {
            return Ok(Outcome::TimedOut(run.last.take()));
        }
        rig.clock.sleep(run.pause(now));
    }
}

/// A round's result: the outcome when the wait is over, `None` to wait on.
type Step<A> = Result<Option<Outcome<<A as Admit>::Held>>, <A as Admit>::Error>;

/// One wait in progress: what the rounds so far have shown.
struct Run<'p> {
    /// The switches.
    pacing: &'p Pacing,
    /// Each signal's history, for the trend in a block reason.
    trends: Trends,
    /// The pause before the next round.
    pause: Duration,
    /// Whether the last round that decided anything failed.
    failing: bool,
    /// When `--timeout` runs out, if it can.
    deadline: Option<Instant>,
    /// The last round's evaluation.
    last: Option<Evaluation>,
}

impl<'p> Run<'p> {
    /// A wait that starts at `start` and has not checked yet.
    fn new(pacing: &'p Pacing, start: Instant) -> Self {
        Self {
            pacing,
            trends: Trends::new(),
            pause: pacing.base(),
            failing: false,
            deadline: pacing.timeout.and_then(|t| start.checked_add(t)),
            last: None,
        }
    }

    /// Whether `--timeout` has run out at `now` (src/wait:V3).
    fn expired(&self, now: Instant) -> bool {
        self.deadline.is_some_and(|deadline| now >= deadline)
    }

    /// Evaluates the gates at `now`; the outcome when the wait is over.
    fn round<S, C, A>(
        &mut self,
        rig: &mut Rig<'_, S, C>,
        now: Instant,
        admit: &mut A,
    ) -> Step<A>
    where
        S: Source + ?Sized,
        C: ?Sized,
        A: Admit,
    {
        let evaluation = evaluate(rig.gates, rig.source, now, &mut self.trends);
        let overall = evaluation.overall;
        self.pace(overall);
        self.last = Some(evaluation);
        if !overall.is_pass() {
            return Ok(None);
        }
        Ok(admit.try_admit()?.map(Outcome::Released))
    }

    /// Sets the next pause from this round's `overall`: a pass resets it,
    /// a failure after a failure backs it off, warming keeps it
    /// (src/wait:V2).
    fn pace(&mut self, overall: Overall) {
        match overall {
            Overall::Pass => {
                self.pause = self.pacing.base();
                self.failing = false;
            }
            Overall::Fail if self.failing => {
                self.pause = self.pacing.backed_off(self.pause);
            }
            Overall::Fail => self.failing = true,
            Overall::Warming | Overall::Unsupported => {}
        }
    }

    /// How long to sleep at `now` before the next round: the pause, cut
    /// short so the last round falls on the deadline.
    fn pause(&self, now: Instant) -> Duration {
        self.deadline.map_or(self.pause, |deadline| {
            self.pause.min(deadline.saturating_duration_since(now))
        })
    }
}

#[cfg(test)]
mod tests;
