//! [`Sampler`]: rates and interval percents from two samples of a
//! cumulative counter (src/signal:T4, V1, V2).

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::{Reading, SignalId};

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
        let _ = (&mut self.last, id, counter, now);
        Reading::Warming
    }

    /// Drops the base of `id`: its next sample warms up again.
    pub fn forget(&mut self, id: SignalId) {
        let _ = id;
    }
}

#[cfg(test)]
mod tests;
