//! Linux thermal sysfs text: zone temperatures, CPU throttle counters,
//! cooling device states (src/power:T1, V1, V2, R3).

use super::PowerError;
use crate::units::Celsius;

/// The most CPUs a `cpu/online` range list may name.
const MAX_CPUS: usize = 65_536;

/// A thermal zone `temp` file: millidegrees Celsius, such as `45000`.
///
/// Rounds down to tenths; below zero reads as `0C`, since a threshold is
/// never negative.
///
/// # Errors
///
/// [`PowerError::Malformed`] when the text is not a whole number.
pub fn parse_millicelsius(text: &str) -> Result<Celsius, PowerError> {
    let milli: i64 = text
        .trim()
        .parse()
        .map_err(|_| PowerError::malformed("temp", text))?;
    let tenths = u32::try_from(milli.max(0).div_euclid(100))
        .map_err(|_| PowerError::malformed("temp", text))?;
    Ok(Celsius::from_tenths(tenths))
}

/// A sysfs counter or state, such as `core_throttle_count` or a cooling
/// device's `cur_state`: one unsigned whole number.
///
/// # Errors
///
/// [`PowerError::Malformed`] naming `file` when it is not one.
pub fn parse_sysfs_u64(
    file: &'static str,
    text: &str,
) -> Result<u64, PowerError> {
    text.trim()
        .parse()
        .map_err(|_| PowerError::malformed(file, text))
}

/// A CPU range list such as `/sys/devices/system/cpu/online`: `0-3,8`.
///
/// # Errors
///
/// [`PowerError::Malformed`] for a part that is not `N` or `N-M` with
/// `N <= M`, or a list of more than 65536 CPUs.
pub fn parse_cpu_list(text: &str) -> Result<Vec<u32>, PowerError> {
    let bad = || PowerError::malformed("cpu list", text);
    let mut cpus = Vec::new();
    for part in text.trim().split(',').filter(|p| !p.is_empty()) {
        let (first, last) = cpu_range(part).ok_or_else(bad)?;
        let span = usize::try_from(last.abs_diff(first)).map_err(|_| bad())?;
        if span >= MAX_CPUS.saturating_sub(cpus.len()) {
            return Err(bad());
        }
        cpus.extend(first..=last);
    }
    Ok(cpus)
}

/// One part of a CPU range list, `N` or `N-M` with `N <= M`.
fn cpu_range(part: &str) -> Option<(u32, u32)> {
    let (first, last) = part.split_once('-').unwrap_or((part, part));
    let (first, last) = (first.parse().ok()?, last.parse().ok()?);
    (first <= last).then_some((first, last))
}

/// Whether a cooling device `type` throttles the CPU: `Processor` (ACPI)
/// or a `cpufreq` cooling device (src/power:R3).
#[must_use]
pub fn is_cpu_cooling(kind: &str) -> bool {
    let kind = kind.trim();
    kind == "Processor" || kind.contains("cpufreq")
}

#[cfg(test)]
mod tests;
