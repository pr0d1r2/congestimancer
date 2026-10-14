//! Sizes: [`Bytes`] with decimal and binary suffixes (src/units:V1, V2).

use std::str::FromStr;

use super::UnitError;

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
    Err(UnitError::Malformed {
        input: s.to_owned(),
        expected: EXPECTED,
    })
}

impl FromStr for Bytes {
    type Err = UnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_bytes(s)
    }
}

#[cfg(test)]
mod tests;
