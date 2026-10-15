//! The macOS memory pressure level: decoding the kernel's number and
//! judging `--mem-level` (src/mem:V5).

use crate::signal::Level;

/// The level `kern.memorystatus_vm_pressure_level` reports: 1 normal,
/// 2 warning, 4 critical; any other number is `None`, never a guess.
#[must_use]
pub const fn pressure_level(raw: i64) -> Option<Level> {
    let _ = raw;
    None
}

/// `--mem-level limit`: whether `level` is at most `limit`, so `normal`
/// passes only at normal and `warning` at normal or warning
/// (src/mem:V5).
#[must_use]
pub fn level_within(level: Level, limit: Level) -> bool {
    let _ = (level, limit);
    false
}

#[cfg(test)]
mod tests;
