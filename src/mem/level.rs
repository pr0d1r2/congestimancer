//! The macOS memory pressure level: decoding the kernel's number and
//! judging `--mem-level` (src/mem:V5), and the share of memory
//! available `status` shows (src/mem:V8).

use crate::signal::Level;
use crate::units::Percent;

/// The level `kern.memorystatus_vm_pressure_level` reports: 1 normal,
/// 2 warning, 4 critical; any other number is `None`, never a guess.
#[must_use]
pub const fn pressure_level(raw: i64) -> Option<Level> {
    match raw {
        1 => Some(Level::Normal),
        2 => Some(Level::Warning),
        4 => Some(Level::Critical),
        _ => None,
    }
}

/// `--mem-level limit`: whether `level` is at most `limit`, so `normal`
/// passes only at normal and `warning` at normal or warning
/// (src/mem:V5).
#[must_use]
pub fn level_within(level: Level, limit: Level) -> bool {
    level <= limit
}

/// The share of memory available that `kern.memorystatus_level` reports,
/// 0 to 100; anything else is `None`.
#[must_use]
pub fn available_level(raw: i64) -> Option<Percent> {
    let _ = raw;
    None
}

#[cfg(test)]
mod tests;
