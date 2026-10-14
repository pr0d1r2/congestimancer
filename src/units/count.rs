//! [`Count`]: a whole number of things (src/units:V9, V14).

use std::str::FromStr;

use super::UnitError;
use super::number::{self, LexError};

/// The form a count takes, named in every error (src/units:V12).
const EXPECTED: &str = "a whole number, like 4";

/// A count of things: processes, slots, jobs.
///
/// Parsed from digits alone: no decimals, no sign, no unit; past `u64` is
/// an error (src/units:V9, V14).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Count(pub u64);

/// Parses a count such as `4`.
///
/// # Errors
///
/// [`UnitError`] naming the input and the form expected: not digits alone,
/// negative, or past `u64`.
pub fn parse_count(s: &str) -> Result<Count, UnitError> {
    let whole = if s.contains('.') {
        Err(LexError::Malformed)
    } else {
        number::lex(s).and_then(|l| l.whole().ok_or(LexError::Malformed))
    };
    whole.map(Count).map_err(|e| e.error(s, EXPECTED))
}

impl FromStr for Count {
    type Err = UnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_count(s)
    }
}

#[cfg(test)]
mod tests;
