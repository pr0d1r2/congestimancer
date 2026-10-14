//! [`Sampler`]: rates and interval percents from two samples of a
//! cumulative counter (src/signal:T4, V1, V2).

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::{Measure, Reading, SignalId};
use crate::units::{Percent, Rate};

/// One sample of a cumulative counter, and what its change over an
/// interval means.
///
/// Domain nodes read the kernel's running totals (bytes read since boot,
/// CPU ticks, busy milliseconds) and hand them over as a `Counter`; the
/// [`Sampler`] turns two of them into a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Counter {
    /// Bytes so far; reads as bytes per second over the interval
    /// (`--io-read`, `--net-rx`, `--swap-rate`).
    Rate {
        /// Running total in bytes.
        total: u64,
    },
    /// Two running totals; reads as the percent the `part` grew of what
    /// the `whole` grew (`--cpu-idle`: idle ticks of all ticks).
    Share {
        /// Running total of the part, such as idle ticks.
        part: u64,
        /// Running total of the whole, such as all ticks.
        whole: u64,
    },
    /// Time spent busy so far; reads as the percent of the interval spent
    /// busy (`--io-busy`).
    Busy {
        /// Running total of busy time.
        busy: Duration,
    },
    /// Time spent on events and how many; reads as the mean time per
    /// event over the interval, zero when none happened (`--io-latency`).
    Mean {
        /// Running total of time spent on events.
        time: Duration,
        /// Running count of events.
        events: u64,
    },
}

/// Keeps the previous [`Counter`] per signal and turns the next one into a
/// [`Reading`].
///
/// The first sample of a signal reads [`Reading::Warming`], never a fake
/// zero (src/signal:V1). A counter that went backwards (a reset, a wrap,
/// a device that came back) or changed kind reads `Warming` again and
/// becomes the new base, never a negative rate (src/signal:V2). A sample
/// at the same instant as the base, or a [`Counter::Share`] whose whole
/// did not move, reads `Warming` and keeps the base.
#[derive(Debug, Default)]
pub struct Sampler {
    /// The base sample per signal: when, and what.
    last: HashMap<SignalId, (Instant, Counter)>,
}

impl Sampler {
    /// A sampler that has seen nothing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes the sample of `id` at monotonic `now` and reads the change
    /// since the base.
    pub fn update(
        &mut self,
        id: SignalId,
        counter: Counter,
        now: Instant,
    ) -> Reading {
        let step = match self.last.get(&id) {
            Some(&(then, before)) => step(before, counter, elapsed(then, now)),
            None => Step::Rebase,
        };
        match step {
            Step::Value(measure) => {
                self.last.insert(id, (now, counter));
                Reading::Value(measure)
            }
            Step::Rebase => {
                self.last.insert(id, (now, counter));
                Reading::Warming
            }
            Step::Wait => Reading::Warming,
        }
    }

    /// Drops the base of `id`: its next sample warms up again.
    pub fn forget(&mut self, id: SignalId) {
        self.last.remove(&id);
    }
}

/// What one more sample makes of the base.
enum Step {
    /// A value; the sample becomes the base.
    Value(Measure),
    /// No value; the sample becomes the base (reset, wrap, new kind).
    Rebase,
    /// No value yet; the base stays (no time or no whole passed).
    Wait,
}

/// The time from `then` to `now`, zero if `now` is not later.
fn elapsed(then: Instant, now: Instant) -> Duration {
    now.checked_duration_since(then).unwrap_or_default()
}

/// What `after` makes of the base `before`, `elapsed` later.
fn step(before: Counter, after: Counter, elapsed: Duration) -> Step {
    let (kind, a0, b0) = totals(before);
    let (same, a1, b1) = totals(after);
    let grown = a1.checked_sub(a0).zip(b1.checked_sub(b0));
    match (kind == same, grown) {
        _ if elapsed.is_zero() => Step::Wait,
        (true, Some((a, b))) => value(kind, (a, b), elapsed.as_nanos()),
        _ => Step::Rebase,
    }
}

/// The kind of counter and its two running totals, in plain numbers:
/// times in nanoseconds, a missing second total as zero.
fn totals(counter: Counter) -> (Kind, u128, u128) {
    match counter {
        Counter::Rate { total } => (Kind::Rate, total.into(), 0),
        Counter::Share { part, whole } => {
            (Kind::Share, part.into(), whole.into())
        }
        Counter::Busy { busy } => (Kind::Busy, busy.as_nanos(), 0),
        Counter::Mean { time, events } => {
            (Kind::Mean, time.as_nanos(), events.into())
        }
    }
}

/// The kinds of [`Counter`], without their totals.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// [`Counter::Rate`].
    Rate,
    /// [`Counter::Share`].
    Share,
    /// [`Counter::Busy`].
    Busy,
    /// [`Counter::Mean`].
    Mean,
}

/// What totals that grew by `(a, b)` over `nanos` read as.
fn value(kind: Kind, (a, b): (u128, u128), nanos: u128) -> Step {
    match kind {
        Kind::Rate => rate(a, nanos),
        Kind::Share if b == 0 => Step::Wait,
        Kind::Share => percent(a, b),
        Kind::Busy => percent(a, nanos),
        Kind::Mean => mean(a, b),
    }
}

/// `bytes` over `nanos` as bytes per second, rounded down.
fn rate(bytes: u128, nanos: u128) -> Step {
    let per_second = bytes
        .checked_mul(NANOS_PER_SECOND)
        .and_then(|n| n.checked_div(nanos))
        .map_or(u64::MAX, |n| u64::try_from(n).unwrap_or(u64::MAX));
    Step::Value(Measure::Rate(Rate(per_second)))
}

/// `part` of `whole` as a percent, rounded down to hundredths, at most
/// 100%.
fn percent(part: u128, whole: u128) -> Step {
    let full = u128::from(Percent::FULL.hundredths());
    let hundredths = part
        .checked_mul(full)
        .and_then(|n| n.checked_div(whole))
        .map_or(full, |n| n.min(full));
    u16::try_from(hundredths)
        .ok()
        .and_then(Percent::from_hundredths)
        .map_or(Step::Wait, |p| Step::Value(Measure::Percent(p)))
}

/// `nanos` spread over `events`, zero when there were none.
fn mean(nanos: u128, events: u128) -> Step {
    let each = nanos.checked_div(events).unwrap_or(0);
    let nanos = u64::try_from(each).unwrap_or(u64::MAX);
    Step::Value(Measure::Duration(Duration::from_nanos(nanos)))
}

/// Nanoseconds in a second.
const NANOS_PER_SECOND: u128 = 1_000_000_000;

#[cfg(test)]
mod tests;
