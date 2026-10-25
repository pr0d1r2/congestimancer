//! [`launch`]: what happens once the wait is over: nothing when there is
//! no command (src/launch:V2), else the command execs (src/launch:V1).

use std::ffi::OsString;
use std::io::Write;
use std::process::Command;

use super::{background, clean_env, exec_command, export_jobs};
use crate::slot::Release;
use crate::sys::inherit_in;

/// How the command starts once released.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Start {
    /// The slot and reservation the command inherits, if any.
    pub release: Option<Release>,
    /// Whether it runs at background priority (`--background`,
    /// `--or-background` after a timeout).
    pub background: bool,
    /// The variables set to the job's share of the CPUs
    /// (`--export-jobs`).
    pub export_jobs: Vec<String>,
    /// The logical CPUs the share is taken of.
    pub threads: u64,
}

impl Start {
    /// A start that hands the command `release`, at background priority
    /// when `background`.
    #[must_use]
    pub const fn new(release: Option<Release>, background: bool) -> Self {
        Self {
            release,
            background,
            export_jobs: Vec::new(),
            threads: 1,
        }
    }

    /// This start, also setting each of `names` to the job's share of
    /// `threads` CPUs (src/launch:V10).
    #[must_use]
    pub fn with_jobs(mut self, names: Vec<String>, threads: u64) -> Self {
        self.export_jobs = names;
        self.threads = threads;
        self
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
    prepare(&start, &mut command, log);
    exec_command(&mut command)
}

/// Readies `command` and this process to start as `start` says: the
/// slot handed over to the command, background priority. A hand-over
/// that fails in the child fails its start, which exits 126.
pub(super) fn prepare(
    start: &Start,
    command: &mut Command,
    log: &mut dyn Write,
) {
    if let Some(release) = &start.release {
        hand_over(release, command);
    }
    if !start.export_jobs.is_empty() {
        let holders = start.release.as_ref().map_or(1, Release::holders);
        export_jobs(command, &start.export_jobs, start.threads, holders);
    }
    if start.background {
        background(log);
    }
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

/// Lets `command` inherit `release`'s slot and reservation locks across
/// exec, so the kernel lets go of them only when it exits (src/slot:V2,
/// V7); cgmr's own descriptors stay close-on-exec, so no other child it
/// starts (a gate's shell, ssh) holds them (src/launch:V9).
fn hand_over(release: &Release, command: &mut Command) {
    if let Some(slot) = release.slot() {
        inherit_in(command, slot.file());
    }
    if let Some(reservation) = release.reservation() {
        inherit_in(command, reservation.file());
    }
}

#[cfg(test)]
mod tests;
