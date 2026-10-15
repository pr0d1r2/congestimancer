//! [`ProcsError`]: a `NAME=N` value that does not parse (src/procs:V4).

use std::error::Error;
use std::fmt;

use crate::units::UnitError;

/// A `--max-procs` or `--max-user-procs` value that does not parse; the
/// CLI prints it and exits 64 (src/procs:V4).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProcsError {
    /// No `=` between pattern and count.
    NoEquals {
        /// The value as given.
        input: String,
    },
    /// Nothing before the `=`.
    EmptyPattern {
        /// The value as given.
        input: String,
    },
    /// The count after the `=` is not a whole number.
    Count(UnitError),
}

impl fmt::Display for ProcsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoEquals { input } => {
                write!(f, "`{input}`: expected PATTERN=N, like cargo=4")
            }
            Self::EmptyPattern { input } => {
                write!(f, "`{input}`: the pattern before `=` is empty")
            }
            Self::Count(err) => err.fmt(f),
        }
    }
}

impl Error for ProcsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Count(err) => Some(err),
            Self::NoEquals { .. } | Self::EmptyPattern { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests;
