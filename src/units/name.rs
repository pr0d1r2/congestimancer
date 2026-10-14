//! [`Name`]: a safe identifier for profiles, slots and custom gates
//! (src/units:V13).

use std::fmt;
use std::str::FromStr;

use super::UnitError;

/// The form a name takes, named in every error (src/units:V12).
const EXPECTED: &str = "a name: 1 to 64 of A-Z a-z 0-9 . _ -, not starting \
                        with - and not . or ..";

/// The longest a name may be, in bytes (all of them ASCII).
const MAX_LEN: usize = 64;

/// A safe identifier.
///
/// One to 64 characters of `A-Z a-z 0-9 . _ -`, not starting with `-` and
/// not `.` or `..` alone, so it is safe as a file name, a path segment and
/// an argv word (src/units:V13).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

impl Name {
    /// The name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Parses a name such as `nightly` or `build-1.v2`.
///
/// # Errors
///
/// [`UnitError`] naming the input and the form expected: empty, a
/// character outside the set, a leading `-`, `.` or `..` alone, or longer
/// than 64.
pub fn parse_name(s: &str) -> Result<Name, UnitError> {
    let input = s.to_owned();
    if !safe(s) {
        return Err(UnitError::Malformed {
            input,
            expected: EXPECTED,
        });
    }
    if s.len() > MAX_LEN {
        return Err(UnitError::OutOfRange {
            input,
            expected: EXPECTED,
        });
    }
    Ok(Name(input))
}

/// Whether `s` is shaped like a name, length aside.
fn safe(s: &str) -> bool {
    let allowed = |c: char| c.is_ascii_alphanumeric() || "._-".contains(c);
    !s.is_empty()
        && s.chars().all(allowed)
        && !s.starts_with('-')
        && s != "."
        && s != ".."
}

impl FromStr for Name {
    type Err = UnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_name(s)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
