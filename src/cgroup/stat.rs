//! [`parse_cpu_stat`]: a cgroup's CPU use and how often its quota stopped
//! it (src/cgroup:T3, V4).

use std::time::Duration;

use super::CgroupError;
use crate::units::Bytes;

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
    let mut fields = Fields::default();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let (key, value) = parse_line(line)?;
        fields.set(key, value);
    }
    let usage = fields.usage.ok_or(CgroupError::Missing {
        file: FILE,
        key: "usage_usec",
    })?;
    Ok(CpuStat {
        usage: Duration::from_micros(usage),
        throttling: fields.throttling(),
    })
}

/// Parses `memory.stat` into the memory a job cannot give back: `anon`,
/// `shmem` and `kernel` (`kernel` is absent before Linux 5.18). Page
/// cache (`file`) is left out, as `memory.peak` includes it and would
/// make every job that reads files look big (src/cgroup:V7).
///
/// # Errors
///
/// [`CgroupError::Malformed`] for a line not `key number`;
/// [`CgroupError::Missing`] without `anon`.
pub fn parse_unreclaimable(text: &str) -> Result<Bytes, CgroupError> {
    let mut anon = None;
    let mut rest: u64 = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let (key, value) = parse_line_in(line, MEMORY_STAT)?;
        match key {
            "anon" => anon = Some(value),
            "shmem" | "kernel" => rest = rest.saturating_add(value),
            _ => {}
        }
    }
    let anon = anon.ok_or(CgroupError::Missing {
        file: MEMORY_STAT,
        key: "anon",
    })?;
    Ok(Bytes(anon.saturating_add(rest)))
}

/// The file [`parse_unreclaimable`] parses.
const MEMORY_STAT: &str = "memory.stat";

/// One `key number` line of `cpu.stat`.
fn parse_line(line: &str) -> Result<(&str, u64), CgroupError> {
    parse_line_in(line, FILE)
}

/// One `key number` line of `file`.
fn parse_line_in<'a>(
    line: &'a str,
    file: &'static str,
) -> Result<(&'a str, u64), CgroupError> {
    let mut words = line.split_whitespace();
    words
        .next()
        .zip(words.next().and_then(|n| n.parse().ok()))
        .filter(|_| words.next().is_none())
        .ok_or_else(|| CgroupError::malformed(file, line))
}

/// The keys of `cpu.stat` this reads, as found.
#[derive(Default)]
struct Fields {
    /// `usage_usec`.
    usage: Option<u64>,
    /// `nr_periods`.
    periods: Option<u64>,
    /// `nr_throttled`.
    throttled_periods: Option<u64>,
    /// `throttled_usec`.
    throttled: Option<u64>,
}

impl Fields {
    /// Keeps `value` if `key` is one this reads.
    fn set(&mut self, key: &str, value: u64) {
        let slot = match key {
            "usage_usec" => &mut self.usage,
            "nr_periods" => &mut self.periods,
            "nr_throttled" => &mut self.throttled_periods,
            "throttled_usec" => &mut self.throttled,
            _ => return,
        };
        *slot = Some(value);
    }

    /// The throttling counters, when all three are there.
    fn throttling(&self) -> Option<Throttling> {
        let (periods, throttled_periods) =
            self.periods.zip(self.throttled_periods)?;
        Some(Throttling {
            periods,
            throttled_periods,
            throttled: Duration::from_micros(self.throttled?),
        })
    }
}

/// The file this parses.
const FILE: &str = "cpu.stat";

#[cfg(test)]
mod tests;
