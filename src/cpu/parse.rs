//! Pure parsers for Linux CPU files (src/cpu:T2, T4).

use super::CpuError;
use crate::units::{Decimal, parse_decimal};

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

#[cfg(test)]
mod tests;
