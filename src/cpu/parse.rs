//! Pure parsers for Linux CPU files (src/cpu:T2, T4).

use super::CpuError;
use crate::units::Decimal;

/// `/proc/loadavg`: the 1, 5 and 15 minute load averages, then tasks.
pub(super) const LOADAVG: &str = "/proc/loadavg";

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
    Err(CpuError::malformed(LOADAVG, text))
}

#[cfg(test)]
mod tests;
