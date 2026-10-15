//! `--rate N/D` and `--spacing D`: how often a key may start, kept as a
//! log of start times in `K+starts` under its `flock` (src/slot:V5, V6).

use std::time::{Duration, SystemTime};

use super::{LockDir, SlotError};
use crate::units::{Name, UnitError};

/// `--rate N/D`: at most `N` starts in any window `D` (src/slot:V5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartRate {
    /// `N`: starts allowed per window, at least 1.
    starts: u64,
    /// `D`: the window, longer than zero.
    window: Duration,
}

impl StartRate {
    /// `N`: starts allowed per window.
    #[must_use]
    pub const fn starts(&self) -> u64 {
        self.starts
    }

    /// `D`: the window.
    #[must_use]
    pub const fn window(&self) -> Duration {
        self.window
    }
}

/// Parses `--rate N/D`, such as `4/1m`: a count and a duration.
///
/// # Errors
///
/// [`UnitError`] naming the input and the form expected: no `/`, a count
/// or duration that does not parse, or either one zero.
pub fn parse_start_rate(s: &str) -> Result<StartRate, UnitError> {
    let _ = s;
    Ok(StartRate {
        starts: 0,
        window: Duration::ZERO,
    })
}

/// What limits a key's starts: `--rate` and `--spacing`, either or both.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StartLimits {
    /// `--rate N/D`.
    pub rate: Option<StartRate>,
    /// `--spacing D`: the least time between two starts.
    pub spacing: Option<Duration>,
}

impl StartLimits {
    /// Whether neither limit is set.
    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.rate.is_none() && self.spacing.is_none()
    }
}

/// The start log after a start at `now_ms` (Unix milliseconds), or `None`
/// when `limits` refuse it. Times past `now_ms` are dropped as bogus: the
/// clock went back, and a log from the future must not wedge the key.
#[must_use]
pub fn admit_start(
    log: &[u64],
    now_ms: u64,
    limits: &StartLimits,
) -> Option<Vec<u64>> {
    let _ = (log, now_ms, limits);
    None
}

/// Records a start for `key` at `now` when `limits` allow it: `true` when
/// it may start, `false` when it must wait (src/slot:V5, V6).
///
/// # Errors
///
/// [`SlotError::Io`] when the start log cannot be read or written.
pub fn record_start(
    dir: &LockDir,
    key: &Name,
    limits: &StartLimits,
    now: SystemTime,
) -> Result<bool, SlotError> {
    let _ = (dir, key, limits, now);
    Ok(false)
}

#[cfg(test)]
mod tests;
