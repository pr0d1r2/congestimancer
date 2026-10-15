//! [`Pacing`]: the timing switches of one wait (src/wait interfaces).

use std::time::Duration;

use crate::report::Style;

/// `--interval` when none is given (src/wait interfaces).
pub const DEFAULT_INTERVAL: Duration = Duration::from_secs(2);

/// The shortest pause between two wake-ups, whatever the switches say,
/// so no wait ever spins (src/wait:V1).
pub const MIN_INTERVAL: Duration = Duration::from_millis(10);

/// How one wait is timed: `--interval`, `--backoff`, `--timeout`,
/// `--settle`, `--jitter`, `--heartbeat`, and `--verbose`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Pacing {
    /// `--interval D`: the pause between rounds.
    pub interval: Duration,
    /// `--backoff D`: the cap a failing interval doubles up to.
    pub backoff: Option<Duration>,
    /// `--timeout D`: give up after this long.
    pub timeout: Option<Duration>,
    /// `--settle D`: how long the gates must pass without a break.
    pub settle: Option<Duration>,
    /// `--jitter D`: the most to sleep at random after release.
    pub jitter: Option<Duration>,
    /// `--heartbeat D`: how often to say why it still waits.
    pub heartbeat: Option<Duration>,
    /// What the `--jitter` pick is drawn from (the caller passes its pid
    /// and start time); the same seed picks the same delay.
    pub seed: u64,
    /// `--verbose`: write each round's block reasons in this style.
    pub verbose: Option<Style>,
}

impl Pacing {
    /// Rounds every `interval`, and nothing else asked for.
    #[must_use]
    pub const fn new(interval: Duration) -> Self {
        Self {
            interval,
            backoff: None,
            timeout: None,
            settle: None,
            jitter: None,
            heartbeat: None,
            seed: 0,
            verbose: None,
        }
    }

    /// The interval as used: never below [`MIN_INTERVAL`] (src/wait:V1).
    pub(super) fn base(&self) -> Duration {
        self.interval.max(MIN_INTERVAL)
    }
}

impl Default for Pacing {
    /// Rounds every [`DEFAULT_INTERVAL`].
    fn default() -> Self {
        Self::new(DEFAULT_INTERVAL)
    }
}

#[cfg(test)]
mod tests;
