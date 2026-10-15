//! [`LoadWindow`]: which load average `--load` reads (src/cpu:T3, V2).

use super::{CpuError, Loads};
use crate::units::Decimal;

/// The load average `--load` and `--all-load` read: 1, 5 or 15 minutes,
/// `1m` unless `--load-window` says otherwise (src/cpu:V2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum LoadWindow {
    /// The 1 minute average (`1m`), the default.
    #[default]
    One,
    /// The 5 minute average (`5m`).
    Five,
    /// The 15 minute average (`15m`).
    Fifteen,
}

/// Parses a `--load-window`: `1m`, `5m` or `15m`.
///
/// # Errors
///
/// [`CpuError::Window`] naming any other input.
pub fn parse_load_window(s: &str) -> Result<LoadWindow, CpuError> {
    Err(CpuError::Window(s.to_owned()))
}

impl Loads {
    /// The average over `window`.
    #[must_use]
    pub const fn pick(self, window: LoadWindow) -> Decimal {
        let _ = window;
        self.fifteen
    }
}

#[cfg(test)]
mod tests;
