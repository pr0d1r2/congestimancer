//! Env cleanup: the command sees none of `cgmr`'s own `CGMR_*` defaults,
//! only the held-slot marker a nested `cgmr` needs (src/launch:V3).

use std::ffi::OsString;
use std::process::Command;

/// Removes from `command` every `CGMR_*` variable in `vars` (the
/// caller's environment) except
/// [`HELD_SLOTS_ENV`](crate::slot::HELD_SLOTS_ENV), which becomes `held`
/// when given: the keys now held, so a nested `cgmr` on the same key does
/// not wait on its parent (src/launch:V3, src/slot:V8).
pub fn clean_env<I>(command: &mut Command, vars: I, held: Option<&str>)
where
    I: IntoIterator<Item = (OsString, OsString)>,
{
    let _ = (command, vars, held);
}

#[cfg(test)]
mod tests;
