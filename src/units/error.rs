//! [`UnitError`]: the input, what was wrong with it, and the form expected
//! (src/units:V12).

use std::fmt;

/// A value given to a switch did not parse.
///
/// Every variant names the input as given and the form expected, so the
/// CLI can print one line and exit 64 (src/units:V12).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitError {
    /// The input is not in the expected form.
    Malformed {
        /// The input as given.
        input: String,
        /// The form expected, as a reader would write it.
        expected: &'static str,
    },
    /// The input is negative; thresholds never are (src/units:V14).
    Negative {
        /// The input as given.
        input: String,
        /// The form expected, as a reader would write it.
        expected: &'static str,
    },
    /// The input is well formed but outside the range its type allows.
    OutOfRange {
        /// The input as given.
        input: String,
        /// The form expected, as a reader would write it.
        expected: &'static str,
    },
    /// The value does not fit; it is never wrapped or saturated
    /// (src/units:V9).
    Overflow {
        /// The input as given.
        input: String,
        /// The form expected, as a reader would write it.
        expected: &'static str,
    },
}

impl UnitError {
    /// The input as given.
    #[must_use]
    pub fn input(&self) -> &str {
        ""
    }

    /// The form expected, as a reader would write it.
    #[must_use]
    pub fn expected(&self) -> &'static str {
        ""
    }
}

impl fmt::Display for UnitError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl std::error::Error for UnitError {}

#[cfg(test)]
mod tests;
