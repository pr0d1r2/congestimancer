//! [`parse_cpu_stat`]: a cgroup's CPU use and how often its quota stopped
//! it (src/cgroup:T3, V4).

use std::time::Duration;

use super::CgroupError;

/// A cgroup's `cpu.stat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct CpuStat {
    /// CPU time used so far (`usage_usec`).
    pub usage: Duration,
    /// Quota enforcement so far; `None` when the cgroup's `cpu`
    /// controller is not enabled, so nothing throttles it.
    pub throttling: Option<Throttling>,
}

/// How often and how long a cgroup's CPU quota stopped it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Throttling {
    /// Enforcement periods so far (`nr_periods`).
    pub periods: u64,
    /// Periods in which the quota ran out (`nr_throttled`).
    pub throttled_periods: u64,
    /// Time spent stopped so far (`throttled_usec`).
    pub throttled: Duration,
}

/// Parses `cpu.stat`: `key value` lines; unknown keys are skipped.
///
/// # Errors
///
/// [`CgroupError::Malformed`] for a line not `key number`;
/// [`CgroupError::Missing`] without `usage_usec`.
pub fn parse_cpu_stat(text: &str) -> Result<CpuStat, CgroupError> {
    Err(CgroupError::malformed(FILE, text))
}

/// The file this parses.
const FILE: &str = "cpu.stat";

#[cfg(test)]
mod tests;
