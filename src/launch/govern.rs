//! `--govern`, the decisions: when the command's process group stops and
//! resumes, and what `cgmr` exits with when the command ends
//! (src/launch:V4). The signalling itself waits on `src/sys`.

use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use crate::gate::Overall;

/// What `--govern` does to the command's process group after a round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GroupAction {
    /// `SIGSTOP`: a gate fails.
    Stop,
    /// `SIGCONT`: the gates pass again.
    Continue,
}

/// The action after a round that said `overall`, for a group that is
/// `stopped` or not; `None` leaves it as it is (src/launch:V4).
#[must_use]
pub const fn govern_step(
    stopped: bool,
    overall: Overall,
) -> Option<GroupAction> {
    match overall {
        Overall::Fail if !stopped => Some(GroupAction::Stop),
        Overall::Pass if stopped => Some(GroupAction::Continue),
        _ => None,
    }
}

/// The exit code that passes on `status`: the command's own, or 128 + n
/// for death by signal n (src/launch:V4).
#[must_use]
pub fn status_code(status: ExitStatus) -> u8 {
    let code = status
        .code()
        .or_else(|| status.signal().and_then(|n| n.checked_add(SIGNAL_BASE)));
    code.and_then(|c| u8::try_from(c).ok()).unwrap_or(u8::MAX)
}

/// What a death by signal n adds n to, as the shell does.
const SIGNAL_BASE: i32 = 128;

#[cfg(test)]
mod tests;
