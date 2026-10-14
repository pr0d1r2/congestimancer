//! [`Celsius`]: a temperature as a number and `C` (src/units:V6).

use std::fmt;
use std::str::FromStr;

use super::UnitError;
use super::number::{self, LexError};

/// The form a temperature takes, named in every error (src/units:V12).
const EXPECTED: &str = "a temperature in degrees Celsius with at most 1 \
                        decimal, like 85C or 72.5C";

/// A temperature threshold in degrees Celsius.
///
/// Kept in tenths of a degree, so it compares exactly; parsed from a
/// number with at most one decimal and a `C` (either case). Thresholds are
/// never negative (src/units:V14).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Celsius(u32);

impl Celsius {
    /// The temperature of `tenths` tenths of a degree.
    #[must_use]
    pub const fn from_tenths(tenths: u32) -> Self {
        Self(tenths)
    }

    /// The temperature in tenths of a degree: `72.5C` is 725.
    #[must_use]
    pub const fn tenths(self) -> u32 {
        self.0
    }

    /// The temperature as a float: `72.5C` is 72.5.
    #[must_use]
    pub fn as_f64(self) -> f64 {
        f64::from(self.0) / 10.0
    }
}

/// Parses a temperature such as `85C` or `72.5C`.
///
/// # Errors
///
/// [`UnitError`] naming the input and the form expected: no `C`,
/// malformed, more than one decimal, negative, or past `u32` tenths.
pub fn parse_celsius(s: &str) -> Result<Celsius, UnitError> {
    let number = s.strip_suffix(['C', 'c']);
    let tenths = number
        .ok_or(LexError::Malformed)
        .and_then(number::lex)
        .and_then(|l| l.scaled(1))
        .and_then(|t| u32::try_from(t).map_err(|_| LexError::Overflow))
        .map_err(|e| e.error(s, EXPECTED))?;
    Ok(Celsius(tenths))
}

impl FromStr for Celsius {
    type Err = UnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_celsius(s)
    }
}

/// Renders with a `C` and no trailing zero: `72.5C`, `85C`.
impl fmt::Display for Celsius {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        number::write_fixed(f, self.0.into(), 1)?;
        f.write_str("C")
    }
}

#[cfg(test)]
mod tests;
