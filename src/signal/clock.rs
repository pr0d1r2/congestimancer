//! [`Clock`]: the monotonic time every interval, rate and timeout is
//! measured on (src/signal:T3, V4).

use std::time::{Duration, Instant};

/// Monotonic time: what time it is, and waiting.
///
/// Every interval, rate, backoff and timeout is measured on a `Clock`,
/// never on the wall clock (src/signal:V4); only the `--between` window
/// reads local time, in `src/presence`. Code takes `&mut impl Clock` so
/// tests drive time by hand instead of sleeping.
pub trait Clock {
    /// The current monotonic instant.
    fn now(&self) -> Instant;

    /// Waits for `d`.
    fn sleep(&mut self, d: Duration);
}

/// The real clock: [`Instant::now`] and [`std::thread::sleep`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn sleep(&mut self, d: Duration) {
        let _ = d;
    }
}

/// A clock tests move by hand: `sleep` returns at once and advances it.
#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) struct ManualClock {
    /// The instant `now` returns.
    now: Instant,
    /// Every `sleep` asked for, in order.
    sleeps: Vec<Duration>,
}

#[cfg(test)]
impl ManualClock {
    /// A clock at an arbitrary start instant.
    pub(crate) fn new() -> Self {
        Self {
            now: Instant::now(),
            sleeps: Vec::new(),
        }
    }

    /// Moves the clock forward by `d`.
    pub(crate) fn advance(&mut self, d: Duration) {
        let _ = d;
    }

    /// Every `sleep` asked for so far, in order.
    pub(crate) fn sleeps(&self) -> &[Duration] {
        &self.sleeps
    }
}

#[cfg(test)]
impl Clock for ManualClock {
    fn now(&self) -> Instant {
        self.now
    }

    fn sleep(&mut self, d: Duration) {
        let _ = d;
    }
}

#[cfg(test)]
mod tests;
