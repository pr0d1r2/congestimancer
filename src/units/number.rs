//! The number lexer every parser shares: `digits[.digits]`, with `.` the
//! only decimal separator and the locale never read (src/units:V8).

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "the parsers that use it land next")
)]

use super::UnitError;

/// A non-negative decimal as written: `int.frac`, `frac` over
/// `10^places`, trailing zeros dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Lexed {
    int: u64,
    frac: u64,
    places: u32,
}

/// Why a number did not lex, before the parser names its form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LexError {
    /// Not `digits[.digits]`.
    Malformed,
    /// A well-formed number behind a `-`.
    Negative,
    /// Does not fit a `u64`.
    Overflow,
}

/// Splits `s` into its leading number (digits, `.` and `-`) and the rest.
pub(super) fn split(s: &str) -> (&str, &str) {
    (s, "")
}

/// Lexes `s` whole as a non-negative decimal.
pub(super) fn lex(_s: &str) -> Result<Lexed, LexError> {
    Err(LexError::Malformed)
}

impl Lexed {
    /// The value, when it has no fractional part.
    pub(super) fn whole(self) -> Option<u64> {
        None
    }

    /// `floor(value * mult)`, or `None` when it does not fit a `u64`.
    pub(super) fn times_floor(self, _mult: u64) -> Option<u64> {
        None
    }

    /// The value in units of `10^-places`; more places than that is
    /// malformed, not rounded.
    pub(super) fn scaled(self, _places: u32) -> Result<u64, LexError> {
        Err(LexError::Malformed)
    }
}

impl LexError {
    /// The [`UnitError`] for `input`, naming the form `expected`.
    pub(super) fn error(
        self,
        input: &str,
        expected: &'static str,
    ) -> UnitError {
        UnitError::Malformed {
            input: input.to_owned(),
            expected,
        }
    }
}

#[cfg(test)]
mod tests;
