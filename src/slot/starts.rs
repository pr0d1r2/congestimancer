//! `--rate N/D` and `--spacing D`: how often a key may start, kept as a
//! log of start times in `K+starts` under its `flock` (src/slot:V5, V6).

use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::file::{lock_wait, open_lock, read_all, rewrite};
use super::{LockDir, SlotError};
use crate::units::{Name, UnitError, parse_count, parse_duration};

/// The form a start rate takes, named in every error (src/units:V12).
const RATE_EXPECTED: &str = "a start rate N/D, like 4/1m";

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
    let malformed = |_| UnitError::Malformed {
        input: s.to_owned(),
        expected: RATE_EXPECTED,
    };
    let (n, d) = s.split_once('/').ok_or(()).map_err(malformed)?;
    let starts = parse_count(n).map_err(|_| malformed(()))?.0;
    let window = parse_duration(d).map_err(|_| malformed(()))?;
    if starts == 0 || window.is_zero() {
        return Err(UnitError::OutOfRange {
            input: s.to_owned(),
            expected: RATE_EXPECTED,
        });
    }
    Ok(StartRate { starts, window })
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
    let past: Vec<u64> = log.iter().copied().filter(|&t| t <= now_ms).collect();
    if !spaced(&past, now_ms, limits.spacing)
        || !under_rate(&past, now_ms, limits.rate)
    {
        return None;
    }
    let mut kept: Vec<u64> = match limits.rate {
        Some(rate) => past
            .into_iter()
            .filter(|&t| within(now_ms, t, rate.window))
            .collect(),
        None => Vec::new(),
    };
    kept.push(now_ms);
    Some(kept)
}

/// `d` in whole milliseconds, saturating.
fn millis(d: Duration) -> u64 {
    u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
}

/// Whether `t` falls in the `window` that ends at `now_ms`.
fn within(now_ms: u64, t: u64, window: Duration) -> bool {
    now_ms.saturating_sub(t) < millis(window)
}

/// Whether the last start in `log` is at least `spacing` before `now_ms`.
fn spaced(log: &[u64], now_ms: u64, spacing: Option<Duration>) -> bool {
    match (log.iter().max(), spacing) {
        (Some(&last), Some(gap)) => now_ms.saturating_sub(last) >= millis(gap),
        _ => true,
    }
}

/// Whether fewer than `N` starts in `log` fall in the window ending at
/// `now_ms`.
fn under_rate(log: &[u64], now_ms: u64, rate: Option<StartRate>) -> bool {
    rate.is_none_or(|r| {
        let recent = log.iter().filter(|&&t| within(now_ms, t, r.window));
        u64::try_from(recent.count()).unwrap_or(u64::MAX) < r.starts
    })
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
    let path = dir.path().join(format!("{key}+starts"));
    let now_ms = now.duration_since(UNIX_EPOCH).map_or(0, millis);
    start_at(&path, limits, now_ms).map_err(|e| SlotError::io(&path, &e))
}

/// [`record_start`] on the start log at `path`, under its `flock`.
fn start_at(
    path: &Path,
    limits: &StartLimits,
    now_ms: u64,
) -> io::Result<bool> {
    let mut file = open_lock(path)?;
    lock_wait(&file)?;
    let text = read_all(&mut file)?;
    let log: Vec<u64> = text
        .split_whitespace()
        .filter_map(|w| w.parse().ok())
        .collect();
    let Some(next) = admit_start(&log, now_ms, limits) else {
        return Ok(false);
    };
    let lines: String = next.iter().map(|t| format!("{t}\n")).collect();
    rewrite(&mut file, &lines)?;
    Ok(true)
}

#[cfg(test)]
mod tests;
