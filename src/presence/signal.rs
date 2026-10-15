//! [`PresenceSignal`]: `--idle` and `--between` (src/presence:T1, T3,
//! T4).

use std::io;
use std::time::Duration;

use super::error::BusError;
use super::logind::logind_idle;
use super::{SystemWallClock, WallClock};
use crate::signal::{Measure, Reason, Sample, Signal, SignalId};
use crate::sys::{current_uid, hid_idle_ns, system_bus_connect};

/// How long a bus reply may take before `--idle` gives up on it.
const BUS_TIMEOUT: Duration = Duration::from_secs(5);

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
    ns.map(Duration::from_nanos).map_err(reason)
}

impl<W: WallClock> PresenceSignal<W> {
    /// `--idle`: time since the last keyboard or mouse input.
    fn idle(&self) -> Result<Duration, Reason> {
        match self.os {
            Os::Macos => hid_idle(hid_idle_ns()),
            Os::Linux => logind_reading(self.linux_idle()),
        }
    }

    /// Linux idle: logind over the system bus `crate::sys` connects.
    fn linux_idle(&self) -> Result<Option<Duration>, BusError> {
        let stream = system_bus_connect()?;
        stream.set_read_timeout(Some(BUS_TIMEOUT))?;
        logind_idle(stream, current_uid(), self.wall.unix_micros())
    }
}

/// Linux idle: logind's answer, or why there is none (V2).
fn logind_reading(
    idle: Result<Option<Duration>, BusError>,
) -> Result<Duration, Reason> {
    idle.map_err(reason)?
        .ok_or_else(|| Reason::from("no local user session to read idle from"))
}

impl<W: WallClock> Signal for PresenceSignal<W> {
    fn sample(&mut self, id: SignalId) -> Sample {
        let sample = match id {
            SignalId::Idle => self.idle().map(Measure::Duration),
            SignalId::Between => self.wall.local_time().map(Measure::Clock),
            _ => Err(Reason::from(format!("{id} is not a presence signal"))),
        };
        sample.map_or_else(Sample::Unsupported, Sample::Gauge)
    }
}

/// An error as the reason a signal has no value.
fn reason(err: impl ToString) -> Reason {
    Reason::from(err.to_string())
}

#[cfg(test)]
mod tests;
