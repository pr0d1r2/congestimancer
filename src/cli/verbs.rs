//! The one-shot verbs: `check`, `status` and `advise`. Each reads the
//! machine once (twice for a rate) and never waits.

use std::time::Duration;

use super::args::{AdviseArgs, Given, Report};
use super::exit::{Failure, check_code};
use super::gates::{Built, Fit, build};
use super::host::{HostSource, machine};
use super::layers::Resolved;
use super::learn::apply;
use super::remote::{Remote, check_on, gate_args};
use super::slots::{learned, reserved};
use super::world::{World, wrote};
use crate::advise::{
    AdviseRequest, advise, advise_exit_code, write_advice, write_advice_json,
};
use crate::cgroup::{CGROUP_SIGNALS, Scoped};
use crate::cpu::cpu_count;
use crate::gate::{Evaluation, Overall, evaluate_once, sample_once};
use crate::remote::{Host, parse_host};
use crate::report::write_status;
use crate::report::{StatusReport, StatusRow, block_reasons, write_json};
use crate::signal::{SignalId, SignalSet, SystemClock};
use crate::units::{parse_bytes, parse_decimal, parse_duration};
use crate::wait::DEFAULT_INTERVAL;

/// `--interval`, or its default: how long a rate's second sample waits.
///
/// # Errors
///
/// A usage error for a value that does not parse.
pub(crate) fn interval(resolved: &Resolved) -> Result<Duration, Failure> {
    let given = resolved.parse("interval", parse_duration)?;
    Ok(given.unwrap_or(DEFAULT_INTERVAL))
}

/// The machine's source for `wanted`, its warnings written.
///
/// # Errors
///
/// A usage error for a bad `--cgroup` or selector.
pub(crate) fn source(
    world: &mut World<'_>,
    resolved: &Resolved,
    wanted: SignalSet,
) -> Result<Scoped<HostSource>, Failure> {
    let mut warnings = Vec::new();
    let root = world.root.clone();
    let found = machine(resolved, &root, wanted, &mut |w| {
        warnings.push(w.to_owned());
    });
    for warning in warnings {
        world.warn(&warning);
    }
    found
}

/// The gates `resolved` asks for, sized for this machine.
///
/// # Errors
///
/// A usage error for a bad value; 69 for an unusable lock dir.
pub(crate) fn gates(
    world: &World<'_>,
    resolved: &Resolved,
    skip: &[&str],
) -> Result<Built, Failure> {
    let needs = resolved.has("needs-mem") && !skip.contains(&"needs-mem");
    let fit = Fit {
        cpus: cpu_count(),
        reserved: reserved(&world.env, needs)?,
    };
    build(resolved, fit, skip)
}

/// The switches of `given` settled, `--needs-mem auto` sized by what was
/// learned for its key (there is no command to name one here).
fn settled(world: &mut World<'_>, given: &Given) -> Result<Resolved, Failure> {
    let mut resolved = world.settle(given)?;
    apply(&world.env, &mut resolved, &[])?;
    Ok(resolved)
}

/// `check`: every gate once; 0 pass, 1 fail, 69 unsupported
/// (src/cli:V4).
///
/// # Errors
///
/// A usage error, or a remote host that cannot evaluate.
pub(crate) fn check(
    world: &mut World<'_>,
    args: &Report,
) -> Result<u8, Failure> {
    let resolved = settled(world, &args.given)?;
    if let Some(host) = resolved.parse("host", parse_host)? {
        return check_host(world, &resolved, host);
    }
    let mut built = gates(world, &resolved, &[])?;
    let mut machine = source(world, &resolved, built.signals)?;
    let (gates, interval) = (&mut built.gates, interval(&resolved)?);
    let eval = evaluate_once(gates, &mut machine, &mut SystemClock, interval);
    if args.json {
        let mut report = StatusReport::new(world.hostname());
        report.gates.clone_from(&eval.verdicts);
        wrote(write_json(world.out, &report))?;
    }
    explain(world, &eval, resolved.on("verbose"));
    Ok(check_code(eval.overall))
}

/// `check --host H`: the gates go to H.
fn check_host(
    world: &mut World<'_>,
    resolved: &Resolved,
    host: Host,
) -> Result<u8, Failure> {
    let mut remote = Remote {
        ssh: &mut *world.ssh,
        host,
        args: gate_args(resolved),
        log: &mut *world.err,
        verbose: resolved.on("verbose"),
    };
    check_on(&mut remote)
}

/// Writes why the gates block when `verbose` or one is unsupported.
fn explain(world: &mut World<'_>, eval: &Evaluation, verbose: bool) {
    if verbose || eval.overall == Overall::Unsupported {
        for reason in block_reasons(eval) {
            world.error(&reason);
        }
    }
}

/// The signals `status` shows: all but the process counts (they need a
/// pattern) and the round trip without `--rtt-host`.
fn shown(resolved: &Resolved) -> SignalSet {
    let probe = resolved.text("rtt-host").is_some();
    let skip = |id: &SignalId| match id {
        SignalId::MaxProcs | SignalId::MaxUserProcs => true,
        SignalId::Rtt => !probe,
        _ => false,
    };
    SignalId::ALL.into_iter().filter(|id| !skip(id)).collect()
}

/// `status`: every signal with its value, as a table or JSON; exit 0.
///
/// # Errors
///
/// A usage error for a bad switch.
pub(crate) fn status(
    world: &mut World<'_>,
    args: &Report,
) -> Result<u8, Failure> {
    let resolved = world.settle(&args.given)?;
    let wanted = shown(&resolved);
    let mut machine = source(world, &resolved, wanted)?;
    let interval = interval(&resolved)?;
    let read = sample_once(&mut machine, wanted, &mut SystemClock, interval);
    let cgroup = machine.scope().cgroup().is_some();
    let report = status_report(world, read, cgroup);
    let written = if args.json {
        write_json(world.out, &report)
    } else {
        write_status(world.out, &report, world.style(world.tty.out))
    };
    wrote(written).map(|()| 0)
}

/// The `status` report of `read`, with what `--needs-mem auto` learned;
/// `cgroup` when the cgroup answers the signals it can.
fn status_report(
    world: &World<'_>,
    read: Vec<(SignalId, crate::signal::Reading)>,
    cgroup: bool,
) -> StatusReport {
    let mut report = StatusReport::new(world.hostname());
    report.signals =
        read.into_iter().map(|(id, r)| row(id, r, cgroup)).collect();
    report.learned = learned(&world.env);
    report
}

/// One `status` row, its source `cgroup` when the cgroup answers it.
fn row(
    id: SignalId,
    reading: crate::signal::Reading,
    cgroup: bool,
) -> StatusRow {
    let row = StatusRow::new(id, reading);
    if cgroup && CGROUP_SIGNALS.contains(&id) {
        row.with_source("cgroup")
    } else {
        row
    }
}

/// The advise request the switches size: `--load` per job, `--needs-mem`
/// per job, `--jobs` wanted (src/advise).
fn request(
    world: &World<'_>,
    resolved: &Resolved,
    jobs: Option<u64>,
) -> Result<AdviseRequest, Failure> {
    let mut request = AdviseRequest::new(cpu_count())
        .with_interval(interval(resolved)?)
        .with_reserved(reserved(&world.env, true)?);
    if let Some(load) = resolved.parse("load", parse_decimal)? {
        request = request.with_load_limit(load);
    }
    if let Some(mem) = resolved.parse("needs-mem", parse_bytes)? {
        request = request.with_needs_mem(mem);
    }
    Ok(jobs.map_or(request, |j| request.with_jobs(j)))
}

/// `advise`: how many jobs fit now; exit 0, or 69 for an unsupported gate.
///
/// # Errors
///
/// A usage error for a bad switch; 69 for an unusable lock dir.
pub(crate) fn advise_verb(
    world: &mut World<'_>,
    args: &AdviseArgs,
) -> Result<u8, Failure> {
    let resolved = settled(world, &args.report.given)?;
    let request = request(world, &resolved, args.jobs)?;
    let mut built = gates(world, &resolved, &["load", "needs-mem"])?;
    let mut wanted = built.signals;
    wanted.insert(SignalId::Load);
    wanted.insert(SignalId::Mem);
    let mut machine = source(world, &resolved, wanted)?;
    let result =
        advise(&request, &mut built.gates, &mut machine, &mut SystemClock);
    match (&result, args.report.json) {
        (Ok(advice), true) => wrote(write_advice_json(world.out, advice))?,
        (Ok(advice), false) => wrote(write_advice(world.out, advice))?,
        (Err(err), _) => world.error(&err.to_string()),
    }
    Ok(advise_exit_code(&result))
}

#[cfg(test)]
mod tests;
