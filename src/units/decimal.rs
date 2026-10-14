//! [`Decimal`]: a load-like threshold that takes decimals, like `0.5`
//! (src/units:V14).

use std::fmt;
use std::str::FromStr;

use super::UnitError;
use super::number;

/// The form a decimal takes, named in every error (src/units:V12).
const EXPECTED: &str = "a non-negative number with at most 3 decimals, like \
                        0.5 or 4";

/// A non-negative load-like threshold, such as a load average per core.
///
/// Kept in thousandths, so it compares exactly; parsed from a number with
/// at most three decimals and no unit. Thresholds are never negative
/// (src/units:V14).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(u64);

impl Decimal {
    /// The decimal of `thousandths` thousandths.
    #[must_use]
    pub const fn from_thousandths(thousandths: u64) -> Self {
        Self(thousandths)
    }

    /// The value in thousandths: `0.5` is 500.
    #[must_use]
    pub const fn thousandths(self) -> u64 {
        self.0
    }

    /// The value as a float: `0.5` is 0.5. Past 2^53 thousandths the
    /// float is the nearest one.
    #[must_use]
    pub fn as_f64(self) -> f64 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a threshold compared as a float; nearest is exact enough"
        )]
        let thousandths = self.0 as f64;
        thousandths / 1000.0
    }
}

/// Parses a decimal such as `0.5` or `4`.
///
/// # Errors
///
/// [`UnitError`] naming the input and the form expected: malformed, more
/// than three decimals, negative, or past `u64` thousandths.
pub fn parse_decimal(s: &str) -> Result<Decimal, UnitError> {
    number::lex(s)
        .and_then(|l| l.scaled(3))
        .map(Decimal)
        .map_err(|e| e.error(s, EXPECTED))
}

impl FromStr for Decimal {
    type Err = UnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_decimal(s)
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
