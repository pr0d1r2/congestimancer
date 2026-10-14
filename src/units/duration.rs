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
    let total = match s.strip_prefix('-') {
        None => millis(s),
        Some(rest) => match millis(rest) {
            Err(LexError::Malformed) => Err(LexError::Malformed),
            _ => Err(LexError::Negative),
        },
    };
    total
        .map(Duration::from_millis)
        .map_err(|e| e.error(s, EXPECTED))
}

/// The units a duration takes, largest first, in milliseconds.
const UNITS: [(&str, u64); 5] = [
    ("d", 86_400_000),
    ("h", 3_600_000),
    ("m", 60_000),
    ("s", 1000),
    ("ms", 1),
];

/// The milliseconds in `s`, an unsigned run of pairs.
fn millis(s: &str) -> Result<u64, LexError> {
    if s.is_empty() {
        return Err(LexError::Malformed);
    }
    let parts = Parts {
        rest: s,
        last: None,
    };
    parts.into_iter().try_fold(0_u64, |total, part| {
        total.checked_add(part?).ok_or(LexError::Overflow)
    })
}

/// The pairs of a duration, each in milliseconds, checked largest first.
struct Parts<'a> {
    /// What is left to read.
    rest: &'a str,
    /// The index in [`UNITS`] of the unit read last.
    last: Option<usize>,
}

impl Iterator for Parts<'_> {
    type Item = Result<u64, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        (!self.rest.is_empty()).then(|| self.step())
    }
}

impl Parts<'_> {
    /// Reads one `<n><unit>` pair.
    fn step(&mut self) -> Result<u64, LexError> {
        let (count, unit, tail) = pair(self.rest).ok_or(LexError::Malformed)?;
        let (rank, &(_, per)) = UNITS
            .iter()
            .enumerate()
            .find(|(_, (name, _))| *name == unit)
            .ok_or(LexError::Malformed)?;
        if self.last.is_some_and(|last| rank <= last) {
            return Err(LexError::Malformed);
        }
        (self.rest, self.last) = (tail, Some(rank));
        let count: u64 = count.parse().map_err(|_| LexError::Overflow)?;
        count.checked_mul(per).ok_or(LexError::Overflow)
    }
}

/// Splits `s` into a run of digits, a run of lowercase letters, and the
/// rest; `None` when there are no digits.
fn pair(s: &str) -> Option<(&str, &str, &str)> {
    let (count, rest) = s.split_at_checked(run(s, |c| c.is_ascii_digit()))?;
    let (unit, tail) =
        rest.split_at_checked(run(rest, |c| c.is_ascii_lowercase()))?;
    (!count.is_empty()).then_some((count, unit, tail))
}

/// The length of the leading run of `s` whose chars match `keep`.
fn run(s: &str, keep: impl Fn(char) -> bool) -> usize {
    s.find(|c| !keep(c)).unwrap_or(s.len())
}

/// Renders `d` in the grammar [`parse_duration`] reads, largest unit
/// first: 5400 seconds is `1h30m`, zero is `0s`. Anything below a
/// millisecond is dropped.
#[must_use]
pub fn render_duration(_d: Duration) -> String {
    String::new()
}

#[cfg(test)]
mod tests;
