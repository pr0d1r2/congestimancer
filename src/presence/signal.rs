//! [`PresenceSignal`]: `--idle` and `--between` (src/presence:T1, T3,
//! T4).

use std::io;
use std::time::Duration;

use super::{SystemWallClock, WallClock};
use crate::signal::{Reason, Sample, Signal, SignalId};

/// Which OS's idle source a [`PresenceSignal`] reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
    /// logind over the system bus.
    Linux,
    /// IOKit `HIDIdleTime` through `crate::sys`.
    Macos,
}

/// The presence signals: `--idle` (time since the last keyboard or mouse
/// input) and `--between` (the local time of day). Wrap it in
/// [`Sampled`](crate::signal::Sampled) for a
/// [`Source`](crate::signal::Source).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresenceSignal<W = SystemWallClock> {
    /// The OS whose idle source is read.
    os: Os,
    /// The wall clock.
    wall: W,
}

impl PresenceSignal {
    /// Reads this OS's sources on the machine's wall clock.
    #[must_use]
    pub fn new() -> Self {
        Self::with_clock(SystemWallClock)
    }
}

impl Default for PresenceSignal {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: WallClock> PresenceSignal<W> {
    /// Reads this OS's sources on `wall`.
    #[must_use]
    pub fn with_clock(wall: W) -> Self {
        let os = if cfg!(target_os = "macos") {
            Os::Macos
        } else {
            Os::Linux
        };
        Self::on(os, wall)
    }

    /// Reads `os`'s sources on `wall`.
    const fn on(os: Os, wall: W) -> Self {
        Self { os, wall }
    }
}

/// macOS idle: `HIDIdleTime` nanoseconds as a duration (src/presence:V1).
fn hid_idle(ns: io::Result<u64>) -> Result<Duration, Reason> {
    let _ = ns;
    Err(Reason::from("stub"))
}

impl<W: WallClock> Signal for PresenceSignal<W> {
    fn sample(&mut self, id: SignalId) -> Sample {
        let _ = (id, self.os, &self.wall, hid_idle);
        Sample::Unsupported(Reason::from("stub"))
    }
}

#[cfg(test)]
mod tests;
