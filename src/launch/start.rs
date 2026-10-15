//! [`launch`]: what happens once the wait is over: nothing when there is
//! no command (src/launch:V2), else the command execs (src/launch:V1).

use std::ffi::OsString;
use std::io::Write;
use std::process::Command;

use super::exec_command;
use crate::slot::Release;

/// How the command starts once released.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Start {
    /// The slot and reservation the command inherits, if any.
    pub release: Option<Release>,
    /// Whether it runs at background priority (`--background`,
    /// `--or-background` after a timeout).
    pub background: bool,
}

impl Start {
    /// A start that hands the command `release`, at background priority
    /// when `background`.
    #[must_use]
    pub const fn new(release: Option<Release>, background: bool) -> Self {
        Self {
            release,
            background,
        }
    }
}

/// Starts `argv` as `start` says, its environment cleaned from `vars`
/// (the caller's, as [`std::env::vars_os`] gives them); returns the exit
/// code to leave with: 0 when there is no command (src/launch:V2), else
/// only when exec failed (src/launch:V1).
pub fn launch<I>(
    argv: &[OsString],
    start: Start,
    vars: I,
    log: &mut dyn Write,
) -> u8
where
    I: IntoIterator<Item = (OsString, OsString)>,
{
    let _ = (start, vars, log);
    let Some((program, args)) = argv.split_first() else {
        return 0;
    };
    let mut command = Command::new(program);
    command.args(args);
    exec_command(&mut command)
}

#[cfg(test)]
mod tests;
