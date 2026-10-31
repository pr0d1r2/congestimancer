//! [`launch_governed`]: `--govern`'s start. The command is spawned, not
//! exec'd, as the leader of its own process group, with `SIGINT`,
//! `SIGTERM` and `SIGHUP` blocked in `cgmr` only from then on, so a
//! signal while waiting still has its default disposition (`.:V22`).
//! `cgmr` holds the slot and reservation until the command ends; the
//! command inherits them too, so a paused command keeps them
//! (src/launch:V4, V8).

use std::ffi::OsString;
use std::io::{self, Write};
use std::process::{Child, Command, ExitStatus};
use std::time::Duration;

use std::os::unix::process::CommandExt as _;

use super::start::{command, prepare};
use super::{
    CrewRules, Crewed, GovernRig, Group, Judge, Start, exec_failure_code,
    govern, wall_nanos,
};
use crate::signal::SystemClock;
use crate::slot::{Crew, LockDir, Release};
use crate::sys::{Blocked, GroupSignal, Wake, block_forwarded, signal_group};
use crate::units::Name;

/// Where `--govern` jobs of one slot key take turns, and by what rules.
#[derive(Debug, Clone)]
pub struct CrewPlan {
    /// The lock dir their crew files live in.
    pub dir: LockDir,
    /// The slot key.
    pub key: Name,
    /// How they take turns.
    pub rules: CrewRules,
}

/// What `--govern` judges each round, how often, and where it writes.
#[non_exhaustive]
pub struct Governing<'a, J: ?Sized> {
    /// What each round says: [`LocalGates`](super::LocalGates), or the
    /// remote gates with `--host`.
    pub judge: &'a mut J,
    /// How often a round runs (`--interval`).
    pub interval: Duration,
    /// Stderr.
    pub log: &'a mut dyn Write,
    /// The crew of the slot key to take turns with, if there is one.
    pub crew: Option<CrewPlan>,
}

impl<'a, J: ?Sized> Governing<'a, J> {
    /// Governing by `judge` every `interval`, writing to `log`.
    pub fn new(
        judge: &'a mut J,
        interval: Duration,
        log: &'a mut dyn Write,
    ) -> Self {
        Self {
            judge,
            interval,
            log,
            crew: None,
        }
    }

    /// These, taking turns with the governed jobs of `plan`'s slot key.
    #[must_use]
    pub fn with_crew(mut self, plan: CrewPlan) -> Self {
        self.crew = Some(plan);
        self
    }
}

/// Starts `argv` as `start` says (environment cleaned from `vars`, slot
/// handed over, background priority) and governs it until it ends;
/// returns the exit code to leave with: 0 with no command, 126 or 127
/// when it cannot start, else the command's own (src/launch:V4).
pub fn launch_governed<I, J>(
    argv: &[OsString],
    start: Start,
    vars: I,
    governing: Governing<'_, J>,
) -> u8
where
    I: IntoIterator<Item = (OsString, OsString)>,
    J: Judge + ?Sized,
{
    let held = start.release.as_ref().and_then(Release::held_marker);
    let Some(mut command) = command(argv, vars, held) else {
        return 0;
    };
    prepare(&start, &mut command, governing.log);
    let code = match Spawned::spawn(&mut command) {
        Ok(mut group) => run(&mut group, governing),
        Err(err) => exec_failure_code(&err),
    };
    // The slot and reservation are let go only once the command ended.
    drop(start);
    code
}

/// Governs `group` as `governing` says, on the real clock.
fn run<J: Judge + ?Sized>(
    group: &mut Spawned,
    governing: Governing<'_, J>,
) -> u8 {
    let Governing {
        judge,
        interval,
        log,
        crew,
    } = governing;
    let mut mates = crew.and_then(|plan| join(&plan, &mut *log));
    let mut rig = GovernRig::new(judge, &SystemClock, group, &mut *log);
    if let Some(mates) = mates.as_mut() {
        rig = rig.with_mates(mates);
    }
    govern(rig, interval).unwrap_or_else(|err| lost(&err, log))
}

/// This job joined to the crew `plan` says; `None`, with why on `log`,
/// when it cannot, and the job then governs alone.
fn join(plan: &CrewPlan, log: &mut dyn Write) -> Option<Crewed> {
    match Crew::join(&plan.dir, &plan.key, wall_nanos()) {
        Ok(crew) => Some(Crewed::new(crew, plan.rules)),
        Err(err) => {
            let _ = writeln!(log, "cgmr: governing alone: {err}");
            None
        }
    }
}

/// The exit code when the command can no longer be followed: 1, with
/// why on `log`.
fn lost(err: &io::Error, log: &mut dyn Write) -> u8 {
    let _ = writeln!(log, "cgmr: lost track of the command: {err}");
    1
}

/// The spawned command, with this thread's forwarded signals blocked.
#[derive(Debug)]
pub(super) struct Spawned {
    /// The command, leader of its own group.
    child: Child,
    /// Forwarded signals, blocked until drop.
    blocked: Blocked,
}

impl Spawned {
    /// Blocks the forwarded signals, then spawns `command` in a group of
    /// its own.
    pub(super) fn spawn(command: &mut Command) -> io::Result<Self> {
        let blocked = block_forwarded()?;
        blocked.unblock_in(command);
        let child = command.process_group(0).spawn()?;
        Ok(Self { child, blocked })
    }
}

impl Group for Spawned {
    fn signal(&mut self, sig: GroupSignal) -> io::Result<()> {
        signal_group(self.child.id(), sig)
    }

    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }

    fn wait(&mut self, d: Duration) -> io::Result<Option<Wake>> {
        self.blocked.wait_timeout(d)
    }
}

#[cfg(test)]
mod tests;
