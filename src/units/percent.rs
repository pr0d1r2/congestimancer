//! [`Percent`]: a number from 0 to 100 and an optional `%`
//! (src/units:V4).

use std::fmt;
use std::str::FromStr;

use super::UnitError;
use super::number;

/// The form a percent takes, named in every error (src/units:V12).
const EXPECTED: &str = "a percent from 0 to 100 with at most 2 decimals, \
                        like 80 or 12.5%";

/// A percentage from 0 to 100.
///
/// Kept in hundredths of a percent, so it compares exactly; parsed from a
/// number with at most two decimals and an optional `%`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Percent(u16);

impl Percent {
    /// The whole: 100%.
    pub const FULL: Self = Self(10_000);

    /// The percent of `hundredths` hundredths, if it is at most 100%.
    #[must_use]
    pub const fn from_hundredths(hundredths: u16) -> Option<Self> {
        if hundredths > Self::FULL.0 {
            return None;
        }
        Some(Self(hundredths))
    }

    /// The percent in hundredths: `12.5%` is 1250.
    #[must_use]
    pub const fn hundredths(self) -> u16 {
        self.0
    }

    /// The percent as a float: `12.5%` is 12.5.
    #[must_use]
    pub fn as_f64(self) -> f64 {
        f64::from(self.0) / 100.0
    }
}

/// Parses a percent such as `80`, `80%` or `12.5%`.
///
/// # Errors
///
/// [`UnitError`] naming the input and the form expected: malformed, more
/// than two decimals, negative, or past 100.
pub fn parse_percent(s: &str) -> Result<Percent, UnitError> {
    let number = s.strip_suffix('%').unwrap_or(s);
    let hundredths = number::lex(number)
        .and_then(|l| l.scaled(2))
        .map_err(|e| e.error(s, EXPECTED))?;
    u16::try_from(hundredths)
        .ok()
        .and_then(Percent::from_hundredths)
        .ok_or_else(|| UnitError::OutOfRange {
            input: s.to_owned(),
            expected: EXPECTED,
        })
}

impl FromStr for Percent {
    type Err = UnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_percent(s)
    }
}

impl fmt::Display for Percent {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
