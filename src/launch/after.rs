//! [`after_wait`]: the wait's [`Outcome`] turned into a [`Start`] or an
//! exit code; `--or-background` starts a timed-out command anyway, at
//! background priority (src/launch:V6, src/wait:V3).

use super::Start;
use crate::slot::Release;
use crate::wait::Outcome;

/// What `--background` and `--or-background` ask; `--background` with
/// `--or-background` is [`Priority::Background`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Priority {
    /// Neither: start at the caller's priority, exit 124 on `--timeout`.
    #[default]
    Normal,
    /// `--background`: always start at background priority.
    Background,
    /// `--or-background`: start at the caller's priority, or on
    /// `--timeout` at background priority instead of exit 124.
    OrBackground,
}

/// How to start after `outcome`, or the exit code when not to: 124 for a
/// timeout without `--or-background`, 69 for an unsupported gate.
///
/// # Errors
///
/// The exit code when the command does not start.
pub fn after_wait(
    outcome: Outcome<Option<Release>>,
    priority: Priority,
) -> Result<Start, u8> {
    let _ = (outcome, priority);
    Err(0)
}

#[cfg(test)]
mod tests;
