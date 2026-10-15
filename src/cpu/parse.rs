//! Pure parsers for Linux CPU files (src/cpu:T2, T4).

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
        let _ = self;
        None
    }

    /// Ticks with nothing to run: `idle` plus `iowait`; `None` past 64
    /// bits.
    #[must_use]
    pub fn idle_total(self) -> Option<u64> {
        let _ = self;
        None
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
    Err(CpuError::malformed(STAT, text))
}

#[cfg(test)]
mod tests;
