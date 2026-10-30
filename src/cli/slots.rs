//! The slot steps from the CLI's side: the per-user lock dir from the
//! environment, the memory other cgmr jobs reserved, and the plan the
//! slot switches make (src/slot).

use std::ffi::OsString;

use super::env::Env;
use super::exit::{Failure, slot_failure};
use super::layers::Resolved;
use crate::launch::Slots;
use crate::report::LearnedPeak;
use crate::slot::{
    HELD_SLOTS_ENV, LockDir, SlotPlan, StartLimits, Waiter, learned_peaks,
    lock_dir_path, parse_start_rate, reserved_total, slot_key,
};
use crate::sys::current_uid;
use crate::units::{Bytes, Name, parse_bytes, parse_count, parse_duration};

/// The switches that need a slot plan even without `--needs-mem`.
const SLOT_SWITCHES: [&str; 5] =
    ["slots", "fifo", "rate", "spacing", "alone-passes"];

/// The per-user lock dir: `$XDG_RUNTIME_DIR/cgmr`, else `$TMPDIR/cgmr`,
/// else `/tmp/cgmr-<uid>`, made 0700 and checked safe (src/slot:V1).
///
/// # Errors
///
/// 69 for an unsafe or unreadable dir.
pub(crate) fn lock_dir(env: &Env) -> Result<LockDir, Failure> {
    let uid = current_uid();
    let runtime = env.var_os("XDG_RUNTIME_DIR");
    let path = lock_dir_path(runtime, env.var_os("TMPDIR"), uid);
    LockDir::open(path, uid).map_err(slot_failure)
}

/// What running cgmr jobs reserved, read when `needed`; nothing else.
///
/// # Errors
///
/// 69 for a lock dir that cannot be used.
pub(crate) fn reserved(env: &Env, needed: bool) -> Result<Bytes, Failure> {
    if !needed {
        return Ok(Bytes(0));
    }
    reserved_total(&lock_dir(env)?).map_err(slot_failure)
}

/// The memory peaks learned per key, for `status`; none when the lock dir
/// cannot be used, since a listing is no reason to fail.
pub(crate) fn learned(env: &Env) -> Vec<LearnedPeak> {
    let Ok(dir) = lock_dir(env) else {
        return Vec::new();
    };
    learned_peaks(&dir)
        .unwrap_or_default()
        .into_iter()
        .map(|l| LearnedPeak::new(l.key, l.peak))
        .collect()
}

/// The slot plan the switches make for `cmd`, or `None` when no slot
/// step is asked for; `--needs-mem` alone reserves only when a key can
/// be made (src/slot:V3).
///
/// # Errors
///
/// 64 for a bad key or value.
pub(crate) fn plan(
    resolved: &Resolved,
    cmd: &[OsString],
) -> Result<Option<SlotPlan>, Failure> {
    let wants = SLOT_SWITCHES.iter().any(|n| resolved.has(n));
    let needs_mem = resolved.parse("needs-mem", parse_bytes)?;
    if !wants && needs_mem.is_none() {
        return Ok(None);
    }
    let Some(key) = key(resolved, cmd, wants)? else {
        return Ok(None);
    };
    let mut plan = SlotPlan::new(key);
    plan.slots = resolved.parse("slots", parse_count)?;
    plan.fifo = resolved.on("fifo");
    plan.alone = resolved.on("alone-passes");
    plan.limits = limits(resolved)?;
    plan.needs_mem = needs_mem;
    Ok(Some(plan))
}

/// The slot key: `--slot-key`, else the command's name; `None` when
/// neither makes one and only `--needs-mem` asked (src/slot:V3).
fn key(
    resolved: &Resolved,
    cmd: &[OsString],
    required: bool,
) -> Result<Option<Name>, Failure> {
    let program = cmd.first().and_then(|c| c.to_str());
    match slot_key(resolved.text("slot-key"), program) {
        Ok(key) => Ok(Some(key)),
        Err(_) if !required => Ok(None),
        Err(err) => Err(slot_failure(err)),
    }
}

/// `--rate` and `--spacing`.
fn limits(resolved: &Resolved) -> Result<StartLimits, Failure> {
    Ok(StartLimits {
        rate: resolved.parse("rate", parse_start_rate)?,
        spacing: resolved.parse("spacing", parse_duration)?,
    })
}

/// The slot steps for `plan` in the lock dir, holding what this process
/// inherited in `CGMR_HELD_SLOTS` (src/slot:V8); none without a plan.
///
/// # Errors
///
/// 69 for a lock dir that cannot be used; 64 for a marker not UTF-8.
pub(crate) fn slots(
    env: &Env,
    plan: Option<SlotPlan>,
) -> Result<Slots, Failure> {
    let Some(plan) = plan else {
        return Ok(Slots::none());
    };
    let marker = env.text(HELD_SLOTS_ENV)?;
    Ok(Slots::new(Waiter::new(lock_dir(env)?, plan, marker)))
}

#[cfg(test)]
mod tests;
