//! `CGMR_HELD_SLOTS`: the slot keys an ancestor `cgmr` holds, so a nested
//! `cgmr` on the same key goes through instead of waiting on its own
//! parent (src/slot:V8).

use crate::units::Name;

/// The variable that carries the held keys, comma separated.
pub const HELD_SLOTS_ENV: &str = "CGMR_HELD_SLOTS";

/// Whether `marker` (the value of [`HELD_SLOTS_ENV`]) names `key`.
#[must_use]
pub fn is_held(marker: Option<&str>, key: &Name) -> bool {
    let _ = (marker, key);
    false
}

/// `marker` with `key` added, for the command a held slot is handed to.
#[must_use]
pub fn mark_held(marker: Option<&str>, key: &Name) -> String {
    let _ = (marker, key);
    String::new()
}

#[cfg(test)]
mod tests;
