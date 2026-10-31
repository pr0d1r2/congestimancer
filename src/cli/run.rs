//! `run`, the default verb: wait until every gate passes and the slot
//! steps admit, then exec the command, or govern it with `--govern`
//! (src/wait, src/launch).

use std::ffi::OsString;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use super::args::RunArgs;
use super::env::Env;
use super::exit::{Failure, slot_failure};
use super::layers::Resolved;
use super::remote::{Remote, gate_args, wait_remote};
use super::slots::{crew, plan, slots};
use super::verbs::{gates, interval, source};
use super::world::World;
use crate::launch::{
    CrewPlan, Governing, Judge, LocalGates, Priority, Slots, after_wait,
    launch, launch_governed,
};
use crate::remote::{Host, parse_host};
use crate::report::block_reasons;
use crate::signal::SystemClock;
use crate::slot::Release;
use crate::units::parse_duration;
use crate::wait::{Outcome, Pacing, Rig, poll};

/// `run`: the command's own exit once it runs (src/launch:V1), 0 when
/// there is none, 124 on `--timeout`, 69 for an unsupported gate
/// (src/cli:V4).
///
/// # Errors
///
/// A usage error, or a slot step or remote host that cannot run.
pub(crate) fn run(
    world: &mut World<'_>,
    args: &RunArgs,
) -> Result<u8, Failure> {
    let resolved = world.settle(&args.given)?;
    let mut slots = slots(&world.env, plan(&resolved, &args.cmd)?)?;
    let start = Start::new(world, &resolved, &args.cmd)?;
    if let Some(host) = resolved.parse("host", parse_host)? {
        return start.remote(world, host, &mut slots);
    }
    let mut built = gates(world, &resolved, &[])?;
    let mut machine = source(world, &resolved, built.signals)?;
    let (gates, err) = (&mut built.gates, &mut *world.err);
    let mut clock = SystemClock;
    let rig = Rig::new(gates, &mut machine, &mut clock, err);
    let outcome = poll(rig, &start.pacing, &mut slots).map_err(slot_failure)?;
    let mut local = LocalGates::new(&mut built.gates, &mut machine);
    Ok(start.launch(Parts::of(world), outcome, &mut local))
}

/// How the wait is timed and the command starts.
struct Start<'c> {
    /// The wait's timing; its interval paces `--govern`.
    pacing: Pacing,
    /// `--background` and `--or-background`.
    priority: Priority,
    /// `--govern`.
    govern: bool,
    /// The crew of governed jobs to take turns with, from `--slot-key`.
    crew: Option<CrewPlan>,
    /// `--verbose`.
    verbose: bool,
    /// The gate switches, for `--host`.
    gate_args: Vec<String>,
    /// The command.
    cmd: &'c [OsString],
}

impl<'c> Start<'c> {
    /// The start the switches ask for `cmd`.
    fn new(
        world: &World<'_>,
        resolved: &Resolved,
        cmd: &'c [OsString],
    ) -> Result<Self, Failure> {
        Ok(Self {
            pacing: pacing(world, resolved)?,
            priority: priority(resolved),
            govern: resolved.on("govern"),
            crew: crew(&world.env, resolved, interval(resolved)?)?,
            verbose: resolved.on("verbose"),
            gate_args: gate_args(resolved),
            cmd,
        })
    }

    /// `run --host H`: the gates are asked of H; the slots and the
    /// command stay here (src/remote:V1). H's reasons go to stderr.
    fn remote(
        &self,
        world: &mut World<'_>,
        host: Host,
        slots: &mut Slots,
    ) -> Result<u8, Failure> {
        let mut log = io::stderr();
        let mut remote = Remote {
            ssh: &mut *world.ssh,
            host,
            args: self.gate_args.clone(),
            log: &mut log,
            verbose: self.verbose,
        };
        let outcome =
            wait_remote(&mut remote, &self.pacing, &mut SystemClock, slots)?;
        let parts = Parts::split(&world.env, &mut *world.out, &mut *world.err);
        Ok(self.launch(parts, outcome, &mut remote))
    }

    /// Says why the wait ended without a release, then execs or governs
    /// the command; returns the exit code.
    fn launch(
        &self,
        parts: Parts<'_>,
        outcome: Outcome<Option<Release>>,
        judge: &mut dyn Judge,
    ) -> u8 {
        explain(parts.err, &outcome);
        let start = match after_wait(outcome, self.priority) {
            Ok(start) => start,
            Err(code) => return code,
        };
        let _ = parts.out.flush();
        if !self.govern {
            return launch(self.cmd, start, parts.vars, parts.err);
        }
        let mut governing =
            Governing::new(judge, self.pacing.interval, parts.err);
        if let Some(plan) = &self.crew {
            governing = governing.with_crew(plan.clone());
        }
        launch_governed(self.cmd, start, parts.vars, governing)
    }
}

/// What a launch writes to and hands the command.
struct Parts<'w> {
    /// stdout, flushed before exec.
    out: &'w mut dyn Write,
    /// stderr.
    err: &'w mut dyn Write,
    /// The environment the command inherits, cleaned by `src/launch`.
    vars: Vec<(OsString, OsString)>,
}

impl<'w> Parts<'w> {
    /// The parts of `world`.
    fn of(world: &'w mut World<'_>) -> Self {
        Self::split(&world.env, &mut *world.out, &mut *world.err)
    }

    /// The parts made of `env`'s variables, `out` and `err`.
    fn split(
        env: &Env,
        out: &'w mut dyn Write,
        err: &'w mut dyn Write,
    ) -> Self {
        let vars = env.vars();
        Self { out, err, vars }
    }
}

/// Writes why a wait ended without a release: the timeout, or the
/// unsupported gates.
fn explain(err: &mut dyn Write, outcome: &Outcome<Option<Release>>) {
    let lines = match outcome {
        Outcome::TimedOut(last) => {
            let why = last.as_ref().map(block_reasons).unwrap_or_default();
            vec![format!("--timeout ran out: {}", why.join("; "))]
        }
        Outcome::Unsupported(eval) => block_reasons(eval),
        Outcome::Released(_) => Vec::new(),
    };
    for line in lines {
        let _ = writeln!(err, "cgmr: {line}");
    }
}

/// `--background` and `--or-background` (src/launch:V6).
fn priority(resolved: &Resolved) -> Priority {
    if resolved.on("background") {
        Priority::Background
    } else if resolved.on("or-background") {
        Priority::OrBackground
    } else {
        Priority::Normal
    }
}

/// The wait's timing from the switches; the `--jitter` seed is the pid
/// and the start time, so two waiters started together scatter.
fn pacing(world: &World<'_>, resolved: &Resolved) -> Result<Pacing, Failure> {
    let d = |name| resolved.parse(name, parse_duration);
    let mut pacing = Pacing::new(interval(resolved)?);
    pacing.backoff = d("backoff")?;
    pacing.timeout = d("timeout")?;
    pacing.settle = d("settle")?;
    pacing.jitter = d("jitter")?;
    pacing.heartbeat = d("heartbeat")?;
    pacing.seed = seed();
    let verbose = resolved.on("verbose");
    pacing.verbose = verbose.then(|| world.style(world.tty.err));
    Ok(pacing)
}

/// This process's id mixed with the time now.
fn seed() -> u64 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH);
    let nanos = now.map_or(0, |d| d.as_nanos());
    u64::from(std::process::id()) ^ (nanos as u64)
}

#[cfg(test)]
mod tests;
