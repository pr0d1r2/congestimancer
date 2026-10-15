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
        false
    }
}

/// Parses `cpu.max`: `max 100000` or `200000 100000`, in microseconds.
///
/// # Errors
///
/// [`CgroupError::Malformed`] for anything but a quota (a number or
/// `max`) and a period above zero.
pub fn parse_cpu_max(text: &str) -> Result<CpuMax, CgroupError> {
    Err(CgroupError::malformed("cpu.max", text))
}

/// Parses `memory.max`: a size in bytes, or `max` (`None`, no limit).
///
/// # Errors
///
/// [`CgroupError::Malformed`] for anything but a number or `max`.
pub fn parse_memory_max(text: &str) -> Result<Option<Bytes>, CgroupError> {
    Err(CgroupError::malformed("memory.max", text))
}

/// Parses `memory.current`: the bytes the cgroup uses now.
///
/// # Errors
///
/// [`CgroupError::Malformed`] for anything but a number.
pub fn parse_memory_current(text: &str) -> Result<Bytes, CgroupError> {
    Err(CgroupError::malformed("memory.current", text))
}

/// Memory left to the cgroup: `max` less `current`, never below zero, and
/// never above what the host has available (src/cgroup:V2).
#[must_use]
pub fn memory_available(
    max: Bytes,
    current: Bytes,
    host: Option<Bytes>,
) -> Bytes {
    let _ = (max, current, host);
    Bytes(1)
}

#[cfg(test)]
mod tests;
