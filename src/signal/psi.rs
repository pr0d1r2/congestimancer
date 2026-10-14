//! [`parse_psi`]: the Linux pressure stall text shared by `--cpu-pressure`,
//! `--mem-pressure`, `--io-pressure` and the cgroup override
//! (src/signal:T8).

use std::fmt;
use std::time::Duration;

use crate::units::Percent;

/// One pressure file: `/proc/pressure/{cpu,memory,io}` or a cgroup's
/// `{cpu,memory,io}.pressure`.
///
/// ```text
/// some avg10=1.53 avg60=0.87 avg300=0.40 total=12345678
/// full avg10=0.00 avg60=0.00 avg300=0.00 total=0
/// ```
///
/// Gates compare `avg10`: `some` for `--cpu-pressure`, `--mem-pressure`,
/// `--io-pressure`; `full` for the `-full` variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Psi {
    /// Time at least one task stalled.
    pub some: PsiLine,
    /// Time all non-idle tasks stalled at once; absent from `cpu` on
    /// kernels before 5.13.
    pub full: Option<PsiLine>,
}

/// One `some` or `full` line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct PsiLine {
    /// Share of the last 10 seconds spent stalled.
    pub avg10: Percent,
    /// Share of the last 60 seconds spent stalled.
    pub avg60: Percent,
    /// Share of the last 300 seconds spent stalled.
    pub avg300: Percent,
    /// Total stall time since boot (or since the cgroup was made).
    pub total: Duration,
}

/// Why pressure text did not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PsiError {
    /// A line is not `some|full avg10=P avg60=P avg300=P total=N`.
    Malformed {
        /// The line as read.
        line: String,
    },
    /// There is no `some` line.
    MissingSome,
    /// A `some` or `full` line appears twice.
    Duplicate {
        /// `some` or `full`.
        kind: &'static str,
    },
}

impl fmt::Display for PsiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { line } => write!(f, "malformed PSI line: {line}"),
            Self::MissingSome => f.write_str("PSI text has no `some` line"),
            Self::Duplicate { kind } => {
                write!(f, "PSI text has two `{kind}` lines")
            }
        }
    }
}

impl std::error::Error for PsiError {}

/// Parses pressure stall text; blank lines are skipped.
///
/// # Errors
///
/// [`PsiError`]: a malformed line (unknown kind, missing, extra or
/// misordered field, a share past 100, a bad total), two lines of one
/// kind, or no `some` line.
pub fn parse_psi(text: &str) -> Result<Psi, PsiError> {
    let _ = text;
    Err(PsiError::MissingSome)
}

#[cfg(test)]
mod tests;
