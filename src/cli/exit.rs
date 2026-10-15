//! Exit codes (src/cli:V4) and [`Failure`]: why cgmr stops before or
//! instead of running the command.

use std::fmt::Display;

use crate::gate::Overall;
use crate::slot::SlotError;

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

/// `check`'s exit for what the gates say together: 0 pass, 1 fail (or
/// still warming), 69 unsupported (src/cli:V4).
pub(crate) const fn check_code(overall: Overall) -> u8 {
    match overall {
        Overall::Pass => OK,
        Overall::Fail | Overall::Warming => FAIL,
        Overall::Unsupported => UNSUPPORTED,
    }
}

/// A slot step that cannot run: 64 for a bad key (src/slot:V10), else 69
/// (an unsafe or unreadable lock dir, src/slot:V1).
pub(crate) fn slot_failure(err: SlotError) -> Failure {
    match err {
        SlotError::BadKey { .. } => Failure::usage(err),
        _ => Failure::unsupported(err),
    }
}

#[cfg(test)]
mod tests;
