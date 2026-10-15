//! Pure parsers for Linux CPU files (src/cpu:T2, T4).

use std::str::SplitWhitespace;

use super::CpuError;
use crate::units::{Decimal, parse_decimal};

/// `/proc/loadavg`: the 1, 5 and 15 minute load averages, then tasks.
pub(super) const LOADAVG: &str = "/proc/loadavg";

/// `/proc/stat`: CPU times since boot and scheduler counts.
pub(super) const STAT: &str = "/proc/stat";

/// The 1, 5 and 15 minute load averages, exact to thousandths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Loads {
    /// Over the last minute.
    pub one: Decimal,
    /// Over the last five minutes.
    pub five: Decimal,
    /// Over the last fifteen minutes.
    pub fifteen: Decimal,
}

/// Parses `/proc/loadavg` for its three load averages.
///
/// # Errors
///
/// [`CpuError::Malformed`] when the line does not start with three
/// non-negative decimals.
pub fn parse_loadavg(text: &str) -> Result<Loads, CpuError> {
    let line = text.lines().next().unwrap_or_default();
    let mut words = line.split_whitespace().map(parse_decimal);
    let mut next = || words.next().and_then(Result::ok);
    match (next(), next(), next()) {
        (Some(one), Some(five), Some(fifteen)) => {
            Ok(Loads { one, five, fifteen })
        }
        _ => Err(CpuError::malformed(LOADAVG, line)),
    }
}

/// All CPUs' cumulative time per state since boot, in clock ticks: the
/// `cpu` line of `/proc/stat`. Columns an older kernel lacks are zero.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct CpuTimes {
    /// In user mode, guests included.
    pub user: u64,
    /// In user mode at raised nice.
    pub nice: u64,
    /// In the kernel.
    pub system: u64,
    /// Idle.
    pub idle: u64,
    /// Idle while IO was outstanding.
    pub iowait: u64,
    /// Serving interrupts.
    pub irq: u64,
    /// Serving softirqs.
    pub softirq: u64,
    /// Stolen by the hypervisor for other guests.
    pub steal: u64,
}

impl CpuTimes {
    /// Every tick counted once: the eight states summed (guest time is
    /// already in `user`); `None` past 64 bits.
    #[must_use]
    pub fn total(self) -> Option<u64> {
        [
            self.user,
            self.nice,
            self.system,
            self.idle,
            self.iowait,
            self.irq,
            self.softirq,
            self.steal,
        ]
        .into_iter()
        .try_fold(0_u64, u64::checked_add)
    }

    /// Ticks with nothing to run: `idle` plus `iowait`; `None` past 64
    /// bits.
    #[must_use]
    pub const fn idle_total(self) -> Option<u64> {
        self.idle.checked_add(self.iowait)
    }
}

/// What `/proc/stat` says about the CPUs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProcStat {
    /// All CPUs' times since boot.
    pub times: CpuTimes,
    /// Tasks running or runnable now (`procs_running`).
    pub procs_running: u64,
}

/// Parses `/proc/stat` for the `cpu` line and `procs_running`.
///
/// # Errors
///
/// [`CpuError::Malformed`] for either line not in its form (the `cpu`
/// line needs at least user, nice, system and idle);
/// [`CpuError::Missing`] when either is absent.
pub fn parse_stat(text: &str) -> Result<ProcStat, CpuError> {
    Ok(ProcStat {
        times: field(text, "cpu", cpu_times)?,
        procs_running: field(text, "procs_running", alone)?,
    })
}

/// The first `/proc/stat` line whose first word is `key`, its other
/// words read by `read`.
fn field<T>(
    text: &str,
    key: &'static str,
    read: fn(SplitWhitespace<'_>) -> Option<T>,
) -> Result<T, CpuError> {
    let mut lines = text.lines().map(|line| (line, line.split_whitespace()));
    let (line, mut words) = lines
        .find(|(_, words)| words.clone().next() == Some(key))
        .ok_or_else(|| missing(key))?;
    words.next();
    read(words).ok_or_else(|| CpuError::malformed(STAT, line))
}

/// A [`CpuError::Missing`] for `key` of `/proc/stat`.
const fn missing(key: &'static str) -> CpuError {
    CpuError::Missing { file: STAT, key }
}

/// The times after `cpu`: at least the first four, past the eighth
/// ignored, absent ones zero.
fn cpu_times(words: SplitWhitespace<'_>) -> Option<CpuTimes> {
    let mut ticks = [0_u64; 8];
    let mut seen = 0_usize;
    for (slot, word) in ticks.iter_mut().zip(words) {
        *slot = word.parse().ok()?;
        seen = seen.saturating_add(1);
    }
    (seen >= 4).then_some(CpuTimes::from(ticks))
}

impl From<[u64; 8]> for CpuTimes {
    /// The `cpu` line's eight columns, in `/proc/stat` order.
    fn from(ticks: [u64; 8]) -> Self {
        let [user, nice, system, idle, iowait, irq, softirq, steal] = ticks;
        Self {
            user,
            nice,
            system,
            idle,
            iowait,
            irq,
            softirq,
            steal,
        }
    }
}

/// A bare count, alone on the rest of its line.
fn alone(mut words: SplitWhitespace<'_>) -> Option<u64> {
    let n = words.next()?.parse().ok()?;
    words.next().is_none().then_some(n)
}

#[cfg(test)]
mod tests;
