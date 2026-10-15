//! [`parse_cpu_max`], [`parse_memory_max`], [`parse_memory_current`] and
//! [`memory_available`]: a cgroup's CPU and memory limits and what is left
//! of them (src/cgroup:T2, V1, V2).

use std::time::Duration;

use super::CgroupError;
use crate::units::Bytes;

/// A cgroup's `cpu.max`: at most `quota` of CPU time per `period`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct CpuMax {
    /// CPU time allowed per period; `None` is `max`, no limit.
    pub quota: Option<Duration>,
    /// The length of one period.
    pub period: Duration,
}

impl CpuMax {
    /// Whether the quota limits the cgroup (`cpu.max` is not `max`).
    #[must_use]
    pub const fn is_limited(&self) -> bool {
        self.quota.is_some()
    }
}

/// Parses `cpu.max`: `max 100000` or `200000 100000`, in microseconds.
///
/// # Errors
///
/// [`CgroupError::Malformed`] for anything but a quota (a number or
/// `max`) and a period above zero.
pub fn parse_cpu_max(text: &str) -> Result<CpuMax, CgroupError> {
    let mut words = text.split_whitespace();
    let quota = words.next().and_then(|w| limit(w).ok());
    let period = words.next().and_then(|w| w.parse().ok());
    quota
        .zip(period.filter(|&p| p > 0))
        .filter(|_| words.next().is_none())
        .map(|(quota, period)| CpuMax {
            quota: quota.map(Duration::from_micros),
            period: Duration::from_micros(period),
        })
        .ok_or_else(|| CgroupError::malformed("cpu.max", text.trim()))
}

/// Parses `memory.max`: a size in bytes, or `max` (`None`, no limit).
///
/// # Errors
///
/// [`CgroupError::Malformed`] for anything but a number or `max`.
pub fn parse_memory_max(text: &str) -> Result<Option<Bytes>, CgroupError> {
    limit(text.trim())
        .map(|max| max.map(Bytes))
        .map_err(|()| CgroupError::malformed("memory.max", text.trim()))
}

/// Parses `memory.current`: the bytes the cgroup uses now.
///
/// # Errors
///
/// [`CgroupError::Malformed`] for anything but a number.
pub fn parse_memory_current(text: &str) -> Result<Bytes, CgroupError> {
    text.trim()
        .parse()
        .map(Bytes)
        .map_err(|_| CgroupError::malformed("memory.current", text.trim()))
}

/// Memory left to the cgroup: `max` less `current`, never below zero, and
/// never above what the host has available (src/cgroup:V2).
#[must_use]
pub fn memory_available(
    max: Bytes,
    current: Bytes,
    host: Option<Bytes>,
) -> Bytes {
    let left = Bytes(max.0.saturating_sub(current.0));
    host.map_or(left, |host| left.min(host))
}

/// A number, or `max` for none.
fn limit(word: &str) -> Result<Option<u64>, ()> {
    match word {
        "max" => Ok(None),
        _ => word.parse().map(Some).map_err(|_| ()),
    }
}

#[cfg(test)]
mod tests;
