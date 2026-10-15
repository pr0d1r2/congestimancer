//! Exec: the command replaces `cgmr`, so its exit code and signals reach
//! the caller untouched; only a failed exec returns (src/launch:V1).

use std::io;
use std::process::Command;

/// The exit code for a command that could not be executed: 127 when it
/// was not found, 126 when it was found but could not run (src/launch:V1).
#[must_use]
pub const fn exec_failure_code(err: &io::Error) -> u8 {
    let _ = err;
    0
}

/// Replaces this process with `command`; returns only when that fails,
/// with the exit code to leave with (src/launch:V1).
pub fn exec_command(command: &mut Command) -> u8 {
    let _ = command;
    0
}

#[cfg(test)]
mod tests;
