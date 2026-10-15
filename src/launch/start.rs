//! [`launch`]: what happens once the wait is over: nothing when there is
//! no command (src/launch:V2), else the command execs (src/launch:V1).

use std::ffi::OsString;
use std::io::Write;
use std::process::Command;

use super::{clean_env, exec_command};
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
    let _ = log;
    let held = start.release.as_ref().and_then(Release::held_marker);
    let Some(mut command) = command(argv, vars, held) else {
        return 0;
    };
    exec_command(&mut command)
}

/// The command for `argv`, its environment cleaned (src/launch:V3), or
/// `None` when there is none.
fn command<I>(argv: &[OsString], vars: I, held: Option<&str>) -> Option<Command>
where
    I: IntoIterator<Item = (OsString, OsString)>,
{
    let (program, args) = argv.split_first()?;
    let mut command = Command::new(program);
    command.args(args);
    clean_env(&mut command, vars, held);
    Some(command)
}

#[cfg(test)]
mod tests;
