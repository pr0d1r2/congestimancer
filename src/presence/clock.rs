//! [`WallClock`]: the wall time `src/presence` reads, injected so tests
//! set it (src/signal:V4, src/presence:T4).

use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::signal::Reason;
use crate::units::ClockTime;

/// Wall time: the one clock only `src/presence` reads (src/signal:V4).
///
/// Every interval elsewhere runs on the monotonic
/// [`Clock`](crate::signal::Clock); this one answers what the time of day
/// is, for `--between`, and what the Unix time is, to age logind's
/// `IdleSinceHint`.
pub trait WallClock {
    /// Microseconds since the Unix epoch, `0` before it.
    fn unix_micros(&self) -> u64;
    /// The local time of day, after any DST change (src/presence:V4).
    ///
    /// # Errors
    ///
    /// The reason when local time cannot be read here.
    fn local_time(&self) -> Result<ClockTime, Reason>;
}

/// The machine's wall clock.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SystemWallClock;

impl WallClock for SystemWallClock {
    fn unix_micros(&self) -> u64 {
        let since = SystemTime::now().duration_since(UNIX_EPOCH);
        since.map_or(0, |d| u64::try_from(d.as_micros()).unwrap_or(u64::MAX))
    }

    fn local_time(&self) -> Result<ClockTime, Reason> {
        let _ = clock_time;
        Err(Reason::from(
            "local time of day is not read on this build yet",
        ))
    }
}

/// The local time of day `read` from `crate::sys`, or why there is none.
fn clock_time(read: io::Result<(u8, u8)>) -> Result<ClockTime, Reason> {
    let _ = read;
    Err(Reason::from("not yet"))
}

#[cfg(test)]
mod tests;
