//! The number lexer every parser shares: `digits[.digits]`, with `.` the
//! only decimal separator and the locale never read (src/units:V8).

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "the parsers that use it land next")
)]

use super::UnitError;

/// The most fractional digits a number may carry, trailing zeros aside:
/// past any unit's precision, and few enough that the arithmetic stays
/// exact in a `u128`.
const MAX_PLACES: usize = 18;

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
    let numeric = |c: char| c.is_ascii_digit() || c == '.' || c == '-';
    let end = s.find(|c| !numeric(c)).unwrap_or(s.len());
    s.split_at_checked(end).unwrap_or((s, ""))
}

/// Lexes `s` whole as a non-negative decimal.
///
/// # Errors
///
/// [`LexError::Negative`] for `-` before anything that would lex,
/// [`LexError::Overflow`] past `u64`, [`LexError::Malformed`] otherwise.
pub(super) fn lex(s: &str) -> Result<Lexed, LexError> {
    match s.strip_prefix('-').map(unsigned) {
        None => unsigned(s),
        Some(Err(LexError::Malformed)) => Err(LexError::Malformed),
        Some(_) => Err(LexError::Negative),
    }
}

/// `digits[.digits]`, no sign.
fn unsigned(s: &str) -> Result<Lexed, LexError> {
    let (int, frac) = s.split_once('.').unwrap_or((s, "0"));
    let int = digits(int)?;
    let (frac, places) = fraction(frac)?;
    Ok(Lexed { int, frac, places })
}

/// A non-empty run of ASCII digits, as a `u64`.
fn digits(s: &str) -> Result<u64, LexError> {
    if !is_digits(s) {
        return Err(LexError::Malformed);
    }
    s.parse().map_err(|_| LexError::Overflow)
}

/// Whether `s` is a non-empty run of ASCII digits; no sign, no locale.
fn is_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// The digits after the `.`, trailing zeros dropped, and how many remain.
fn fraction(s: &str) -> Result<(u64, u32), LexError> {
    let kept = s.trim_end_matches('0');
    if !is_digits(s) || kept.len() > MAX_PLACES {
        return Err(LexError::Malformed);
    }
    let places = u32::try_from(kept.len()).map_err(|_| LexError::Malformed)?;
    Ok((kept.parse().unwrap_or(0), places))
}

impl Lexed {
    /// The value, when it has no fractional part.
    pub(super) fn whole(self) -> Option<u64> {
        (self.places == 0).then_some(self.int)
    }

    /// `floor(value * mult)`, or `None` when it does not fit a `u64`.
    pub(super) fn times_floor(self, mult: u64) -> Option<u64> {
        let scale = 10_u128.checked_pow(self.places)?;
        let part = u128::from(self.frac)
            .checked_mul(u128::from(mult))?
            .checked_div(scale)?;
        let part = u64::try_from(part).ok()?;
        self.int.checked_mul(mult)?.checked_add(part)
    }

    /// The value in units of `10^-places`; more places than that is
    /// malformed, not rounded.
    pub(super) fn scaled(self, places: u32) -> Result<u64, LexError> {
        if self.places > places {
            return Err(LexError::Malformed);
        }
        let mult = 10_u64.checked_pow(places).ok_or(LexError::Overflow)?;
        self.times_floor(mult).ok_or(LexError::Overflow)
    }
}

impl LexError {
    /// The [`UnitError`] for `input`, naming the form `expected`.
    pub(super) fn error(
        self,
        input: &str,
        expected: &'static str,
    ) -> UnitError {
        let input = input.to_owned();
        match self {
            Self::Malformed => UnitError::Malformed { input, expected },
            Self::Negative => UnitError::Negative { input, expected },
            Self::Overflow => UnitError::Overflow { input, expected },
        }
    }
}

#[cfg(test)]
mod tests;
