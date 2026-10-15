//! [`Run`]: one wait in progress, the state behind
//! [`poll`](super::poll): when the next round and heartbeat fall, how
//! long the gates have passed, what the last round said (src/wait:T1-T7).

use std::io::Write;
use std::time::{Duration, Instant};

use super::{Admit, MIN_INTERVAL, Outcome, Pacing, Rig};
use crate::gate::{Evaluation, Overall, evaluate};
use crate::report::{heartbeat_line, write_verbose};
use crate::signal::{Source, Trends};

/// A round's result: the outcome when the wait is over, `None` to wait.
pub(super) type Step<A> =
    Result<Option<Outcome<<A as Admit>::Held>>, <A as Admit>::Error>;

/// One wait in progress: what the rounds so far have shown.
pub(super) struct Run<'p> {
    /// The switches.
    pacing: &'p Pacing,
    /// When the wait began.
    start: Instant,
    /// Each signal's history, for the trend in a block reason.
    trends: Trends,
    /// The pause after the last round, backed off or not.
    pause: Duration,
    /// Whether the last round that decided anything failed.
    failing: bool,
    /// When `--timeout` runs out, if it can.
    deadline: Option<Instant>,
    /// Since when the gates have passed without a break.
    since: Option<Instant>,
    /// When the next round is due; `None` is now.
    next_round: Option<Instant>,
    /// When the next heartbeat is due, if any.
    next_beat: Option<Instant>,
    /// The last round's evaluation.
    last: Option<Evaluation>,
}

impl<'p> Run<'p> {
    /// A wait that starts at `start` and has not checked yet.
    pub(super) fn new(pacing: &'p Pacing, start: Instant) -> Self {
        Self {
            pacing,
            start,
            trends: Trends::new(),
            pause: pacing.base(),
            failing: false,
            deadline: pacing.timeout.and_then(|t| start.checked_add(t)),
            since: None,
            next_round: None,
            next_beat: beat_after(pacing, start),
            last: None,
        }
    }

    /// Whether a round is due at `now`.
    pub(super) fn due(&self, now: Instant) -> bool {
        self.next_round.is_none_or(|at| now >= at)
    }

    /// Whether `--timeout` has run out at `now` (src/wait:V3).
    pub(super) fn expired(&self, now: Instant) -> bool {
        self.deadline.is_some_and(|deadline| now >= deadline)
    }

    /// The last round's evaluation, taken out.
    pub(super) const fn take_last(&mut self) -> Option<Evaluation> {
        self.last.take()
    }

    /// Evaluates the gates at `now` and, once they have passed and
    /// settled, tries the slot steps; the outcome when the wait is over.
    pub(super) fn round<S, C, A>(
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
        if overall == Overall::Unsupported {
            return Ok(Some(Outcome::Unsupported(evaluation)));
        }
        self.explain(rig.log, &evaluation);
        self.pace(overall);
        let ready = self.settled(overall, now);
        self.last = Some(evaluation);
        self.schedule(now);
        if !ready {
            return Ok(None);
        }
        Ok(admit.try_admit()?.map(Outcome::Released))
    }

    /// With `--verbose`, writes why this round blocks (src/report:V4).
    fn explain(&self, log: &mut dyn Write, evaluation: &Evaluation) {
        if let Some(style) = self.pacing.verbose {
            // Stderr gone is no reason to stop waiting.
            let _ = write_verbose(log, evaluation, style);
        }
    }

    /// With `--heartbeat`, writes the heartbeat line once one is due at
    /// `now` (src/wait:V6).
    pub(super) fn beat(&mut self, now: Instant, log: &mut dyn Write) {
        let (Some(at), Some(last)) = (self.next_beat, &self.last) else {
            return;
        };
        if now < at {
            return;
        }
        let waited = now.saturating_duration_since(self.start);
        // Stderr gone is no reason to stop waiting.
        let _ = writeln!(log, "{}", heartbeat_line(last, waited));
        self.next_beat = beat_after(self.pacing, now);
    }

    /// Whether the gates have passed without a break for `--settle` at
    /// `now`, given this round's `overall`; anything but a pass starts the
    /// span again (src/wait:V4).
    fn settled(&mut self, overall: Overall, now: Instant) -> bool {
        if !overall.is_pass() {
            self.since = None;
            return false;
        }
        let since = *self.since.get_or_insert(now);
        let hold = self.pacing.settle.unwrap_or_default();
        now.saturating_duration_since(since) >= hold
    }

    /// When the current pass will have held for `--settle`.
    fn settles_at(&self) -> Option<Instant> {
        self.since?.checked_add(self.pacing.settle?)
    }

    /// Sets the pause from this round's `overall`: a pass resets it, a
    /// failure after a failure backs it off, warming keeps it
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

    /// Sets the next round one pause after `now`, sooner when the
    /// deadline or the end of settle falls first.
    fn schedule(&mut self, now: Instant) {
        let next = now.checked_add(self.pause);
        let sooner = [self.deadline, self.settles_at()]
            .into_iter()
            .flatten()
            .filter(|&at| at > now);
        self.next_round = earliest(next, sooner);
    }

    /// How long to sleep at `now`: until the next round, heartbeat or
    /// deadline, whichever is first; never zero (src/wait:V1).
    pub(super) fn pause(&self, now: Instant) -> Duration {
        [self.next_round, self.next_beat, self.deadline]
            .into_iter()
            .flatten()
            .filter(|&at| at > now)
            .min()
            .map_or(self.pause, |at| at.saturating_duration_since(now))
    }
}

/// The heartbeat due one `--heartbeat` after `from`, if asked for.
fn beat_after(pacing: &Pacing, from: Instant) -> Option<Instant> {
    let every = pacing.heartbeat?.max(MIN_INTERVAL);
    from.checked_add(every)
}

/// The earliest of `first` (`None`: never) and the `others`.
fn earliest(
    first: Option<Instant>,
    others: impl IntoIterator<Item = Instant>,
) -> Option<Instant> {
    others
        .into_iter()
        .fold(first, |best, at| Some(best.map_or(at, |best| best.min(at))))
}

#[cfg(test)]
mod tests;
