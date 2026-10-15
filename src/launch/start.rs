//! [`launch`]: what happens once the wait is over: nothing when there is
//! no command (src/launch:V2), else the command execs (src/launch:V1).

use std::ffi::OsString;
use std::io::{self, Write};
use std::process::Command;

use super::{background, clean_env, exec_command};
use crate::slot::Release;
use crate::sys::set_inheritable;

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
    let held = start.release.as_ref().and_then(Release::held_marker);
    let Some(mut command) = command(argv, vars, held) else {
        return 0;
    };
    if let Err(code) = prepare(&start, &mut command, log) {
        return code;
    }
    exec_command(&mut command)
}

/// Readies this process to start the command as `start` says: the slot
/// handed over, background priority; the exit code (126) when the slot
/// cannot be handed over.
pub(super) fn prepare(
    start: &Start,
    command: &mut Command,
    log: &mut dyn Write,
) -> Result<(), u8> {
    if let Some(Err(err)) =
        start.release.as_ref().map(|r| hand_over(r, command))
    {
        // The exit code says it; a lost stderr changes nothing.
        let _ = writeln!(log, "cgmr: cannot hand the slot over: {err}");
        return Err(126);
    }
    if start.background {
        background(log);
    }
    Ok(())
}

/// The command for `argv`, its environment cleaned (src/launch:V3), or
/// `None` when there is none.
pub(super) fn command<I>(
    argv: &[OsString],
    vars: I,
    held: Option<&str>,
) -> Option<Command>
where
    I: IntoIterator<Item = (OsString, OsString)>,
{
    let (program, args) = argv.split_first()?;
    let mut command = Command::new(program);
    command.args(args);
    clean_env(&mut command, vars, held);
    Some(command)
}

/// Lets the command inherit `release`'s slot and reservation locks
/// across exec, so the kernel lets go of them only when it exits
/// (src/slot:V2, V7).
fn hand_over(release: &Release, command: &mut Command) -> io::Result<()> {
    let _ = command;
    if let Some(slot) = release.slot() {
        set_inheritable(slot.file())?;
    }
    if let Some(reservation) = release.reservation() {
        set_inheritable(reservation.file())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
