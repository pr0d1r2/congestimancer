//! Env cleanup: the command sees none of `cgmr`'s own `CGMR_*` defaults,
//! only the held-slot marker a nested `cgmr` needs (src/launch:V3).

use std::ffi::OsString;
use std::process::Command;

use crate::slot::HELD_SLOTS_ENV;

/// What every one of `cgmr`'s own variables starts with.
const PREFIX: &str = "CGMR_";

/// Removes from `command` every `CGMR_*` variable in `vars` (the
/// caller's environment) except
/// [`HELD_SLOTS_ENV`](crate::slot::HELD_SLOTS_ENV), which becomes `held`
/// when given: the keys now held, so a nested `cgmr` on the same key does
/// not wait on its parent (src/launch:V3, src/slot:V8).
pub fn clean_env<I>(command: &mut Command, vars: I, held: Option<&str>)
where
    I: IntoIterator<Item = (OsString, OsString)>,
{
    for (name, _) in vars {
        let ours = name.as_encoded_bytes().starts_with(PREFIX.as_bytes());
        if ours && name != HELD_SLOTS_ENV {
            command.env_remove(name);
        }
    }
    if let Some(held) = held {
        command.env(HELD_SLOTS_ENV, held);
    }
}

/// Sets each of `names` in `command` to this job's share of `threads`
/// CPUs when `holders` jobs hold the slot key: `threads / holders`, at
/// least 1 (src/launch:V10). The held-slot marker is never overwritten:
/// a nested `cgmr` needs it to see the slot its parent holds.
pub(super) fn export_jobs(
    command: &mut Command,
    names: &[String],
    threads: u64,
    holders: u64,
) {
    let share = threads.checked_div(holders).unwrap_or(threads).max(1);
    for name in names.iter().filter(|name| *name != HELD_SLOTS_ENV) {
        command.env(name, share.to_string());
    }
}

#[cfg(test)]
mod tests;
