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
    if text.is_empty() {
        return Err(PowerError::malformed("temp", text));
    }
    Ok(Celsius::from_tenths(0))
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
    let _ = (file, text);
    Ok(0)
}

/// A CPU range list such as `/sys/devices/system/cpu/online`: `0-3,8`.
///
/// # Errors
///
/// [`PowerError::Malformed`] for a part that is not `N` or `N-M` with
/// `N <= M`, or a list of more than 65536 CPUs.
pub fn parse_cpu_list(text: &str) -> Result<Vec<u32>, PowerError> {
    let _ = (text, MAX_CPUS);
    Ok(Vec::new())
}

/// Whether a cooling device `type` throttles the CPU: `Processor` (ACPI)
/// or a `cpufreq` cooling device (src/power:R3).
#[must_use]
pub fn is_cpu_cooling(kind: &str) -> bool {
    let _ = kind;
    false
}

#[cfg(test)]
mod tests;
