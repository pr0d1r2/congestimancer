//! Sizes: [`Bytes`] with decimal and binary suffixes (src/units:V1, V2).

use std::fmt;
use std::str::FromStr;

use super::UnitError;
use super::number::{self, LexError};

/// The form a size takes, named in every error (src/units:V12).
const EXPECTED: &str = "a size: a number and B, KB, MB, GB, TB, KiB, MiB, \
                        GiB or TiB, like 10GB or 1.5GiB";

/// A size in bytes.
///
/// Parsed from a number and a unit: `B` is one byte, `KB` `MB` `GB` `TB`
/// are powers of 1000, `KiB` `MiB` `GiB` `TiB` powers of 1024; the unit is
/// case insensitive and there are no bit units (src/units:V1). The number
/// may be fractional and the result rounds down to whole bytes
/// (src/units:V2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bytes(pub u64);

/// Parses a size such as `10GB`, `512MiB` or `1.5GiB`.
///
/// # Errors
///
/// [`UnitError`] naming the input and the form expected: malformed, no
/// unit or an unknown one, negative, or past `u64` bytes (src/units:V9).
pub fn parse_bytes(s: &str) -> Result<Bytes, UnitError> {
    size(s, s, EXPECTED).map(Bytes)
}

/// The units a size takes and their multipliers, largest first.
const UNITS: [(&str, u64); 9] = [
    ("TiB", 1_099_511_627_776),
    ("TB", 1_000_000_000_000),
    ("GiB", 1_073_741_824),
    ("GB", 1_000_000_000),
    ("MiB", 1_048_576),
    ("MB", 1_000_000),
    ("KiB", 1024),
    ("KB", 1000),
    ("B", 1),
];

/// The bytes in `s`, a number and a unit; errors name `input` and
/// `expected`, which a caller wrapping the size may widen.
fn size(
    s: &str,
    input: &str,
    expected: &'static str,
) -> Result<u64, UnitError> {
    let (number, unit) = number::split(s);
    let lexed = number::lex(number).map_err(|e| e.error(input, expected))?;
    let mult = multiplier(unit)
        .ok_or_else(|| LexError::Malformed.error(input, expected))?;
    lexed
        .times_floor(mult)
        .ok_or_else(|| LexError::Overflow.error(input, expected))
}

/// The multiplier of a unit, matched case insensitively.
fn multiplier(unit: &str) -> Option<u64> {
    UNITS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(unit))
        .map(|&(_, mult)| mult)
}

impl FromStr for Bytes {
    type Err = UnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_bytes(s)
    }
}

/// The form a rate takes, named in every error (src/units:V12).
const RATE_EXPECTED: &str = "a rate: a size per second, like 50MB/s or \
                             1.5GiB/s";

/// A rate in bytes per second.
///
/// Parsed from a size and `/s`; there is no other time base
/// (src/units:V3).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rate(pub u64);

/// Parses a rate such as `50MB/s` or `1.5GiB/s`.
///
/// # Errors
///
/// [`UnitError`] naming the whole input and the form expected: no `/s`,
/// or a size that does not parse (see [`parse_bytes`]).
pub fn parse_rate(s: &str) -> Result<Rate, UnitError> {
    let per_second = s.strip_suffix("/s").or_else(|| s.strip_suffix("/S"));
    let size_part = per_second
        .ok_or_else(|| LexError::Malformed.error(s, RATE_EXPECTED))?;
    size(size_part, s, RATE_EXPECTED).map(Rate)
}

impl FromStr for Rate {
    type Err = UnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_rate(s)
    }
}

/// Renders in the largest unit that divides exactly, so it reads back
/// unchanged: `1KiB`, `1536B`, `0B`.
impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.0;
        let (count, unit) = UNITS
            .iter()
            .filter(|_| n != 0)
            .find(|&&(_, mult)| n.checked_rem(mult) == Some(0))
            .and_then(|&(unit, mult)| Some((n.checked_div(mult)?, unit)))
            .unwrap_or((n, "B"));
        write!(f, "{count}{unit}")
    }
}

/// Renders as its size per second: `50MB/s`.
impl fmt::Display for Rate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/s", Bytes(self.0))
    }
}

#[cfg(test)]
mod tests;
