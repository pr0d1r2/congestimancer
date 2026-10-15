//! Exit codes (src/cli:V4) and [`Failure`]: why cgmr stops before or
//! instead of running the command.

use std::fmt::Display;

/// Every gate passed and there is no command; `check`: every gate passed.
pub(crate) const OK: u8 = 0;
/// `check`: a gate failed.
pub(crate) const FAIL: u8 = 1;
/// A usage error: clap, a value that does not parse, a bad profile.
pub(crate) const USAGE: u8 = 64;
/// A gate has no source on this machine, or cgmr cannot work here.
pub(crate) const UNSUPPORTED: u8 = 69;

/// Why cgmr stops: the exit code and the message for stderr.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Failure {
    /// The exit code.
    pub(crate) code: u8,
    /// What went wrong, without the `cgmr:` prefix.
    pub(crate) message: String,
}

impl Failure {
    /// A usage error, exit 64.
    pub(crate) fn usage(message: impl Display) -> Self {
        Self::new(USAGE, message)
    }

    /// Cannot work here, exit 69.
    pub(crate) fn unsupported(message: impl Display) -> Self {
        Self::new(UNSUPPORTED, message)
    }

    /// Exit `code` because of `message`.
    pub(crate) fn new(code: u8, message: impl Display) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }
}

#[cfg(test)]
mod tests;
