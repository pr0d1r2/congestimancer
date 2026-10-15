//! [`ProcLimit`] and [`parse_proc_limit`]: the `NAME=N` / `USER=N` value
//! of `--max-procs` and `--max-user-procs` (src/procs:T2, V4).

use std::fmt;
use std::str::FromStr;

use super::{Glob, ProcsError};
use crate::units::{Count, parse_count};

/// A pattern and the most processes it may match: `cargo=4`,
/// `nixbld*=8`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ProcLimit {
    /// The process or user name pattern.
    pub pattern: Glob,
    /// The most matching processes the gate allows.
    pub max: Count,
}

/// Parses `PATTERN=N`: a non-empty glob, `=`, a whole number. The last
/// `=` splits, so a pattern may hold one.
///
/// # Errors
///
/// [`ProcsError`] for a value with no `=`, an empty pattern, or a bad
/// count; the CLI prints it and exits 64 (src/procs:V4).
pub fn parse_proc_limit(s: &str) -> Result<ProcLimit, ProcsError> {
    let input = || s.to_owned();
    let (pattern, count) = s
        .rsplit_once('=')
        .ok_or_else(|| ProcsError::NoEquals { input: input() })?;
    if pattern.is_empty() {
        return Err(ProcsError::EmptyPattern { input: input() });
    }
    let max = parse_count(count).map_err(ProcsError::Count)?;
    Ok(ProcLimit {
        pattern: Glob::new(pattern),
        max,
    })
}

impl FromStr for ProcLimit {
    type Err = ProcsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_proc_limit(s)
    }
}

/// Renders as the switch value: `nixbld*=8`.
impl fmt::Display for ProcLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.pattern, self.max)
    }
}

#[cfg(test)]
mod tests;
