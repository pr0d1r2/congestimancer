//! `CGMR_HELD_SLOTS`: the slot keys an ancestor `cgmr` holds, so a nested
//! `cgmr` on the same key goes through instead of waiting on its own
//! parent (src/slot:V8).

use crate::units::Name;

/// The variable that carries the held keys, comma separated.
pub const HELD_SLOTS_ENV: &str = "CGMR_HELD_SLOTS";

/// Whether `marker` (the value of [`HELD_SLOTS_ENV`]) names `key`.
#[must_use]
pub fn is_held(marker: Option<&str>, key: &Name) -> bool {
    marker.is_some_and(|m| m.split(',').any(|k| k == key.as_str()))
}

/// `marker` with `key` added, for the command a held slot is handed to.
#[must_use]
pub fn mark_held(marker: Option<&str>, key: &Name) -> String {
    match marker {
        _ if is_held(marker, key) => marker.unwrap_or_default().to_owned(),
        Some(m) if !m.is_empty() => format!("{m},{key}"),
        _ => key.as_str().to_owned(),
    }
}

#[cfg(test)]
mod tests;
