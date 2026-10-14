//! Durations in the crate's own grammar: `<n><unit>` pairs, largest unit
//! first, like `1h30m` (src/units:V5); no `humantime`.

use std::time::Duration;

use super::UnitError;
use super::number::LexError;

/// The form a duration takes, named in every error (src/units:V12).
const EXPECTED: &str = "a duration: whole numbers each with a unit d, h, m, \
                        s or ms, largest first, like 1h30m or 500ms";

/// Parses a duration such as `1h30m`, `90s` or `500ms` into a [`Duration`]
/// of whole milliseconds.
///
/// # Errors
///
/// [`UnitError`] naming the input and the form expected: a bare number, an
/// unknown or repeated unit, units out of order, negative, or past `u64`
/// milliseconds (src/units:V9).
pub fn parse_duration(s: &str) -> Result<Duration, UnitError> {
    Err(LexError::Malformed.error(s, EXPECTED))
}

#[cfg(test)]
mod tests;
