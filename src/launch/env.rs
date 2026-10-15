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

#[cfg(test)]
mod tests;
