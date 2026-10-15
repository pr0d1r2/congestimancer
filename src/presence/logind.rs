//! Linux idle from logind: every session's idle facts over the system
//! bus, then the least idle local user session (src/presence:T2, V2, V5,
//! R1).

use std::io::{Read, Write};
use std::time::Duration;

use super::error::BusError;

/// One logind session's idle facts (R1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Session {
    /// `Class`: `user`, `greeter`, `lock-screen`, `background`, ...
    pub(super) class: String,
    /// `Remote`: logged in over the network.
    pub(super) remote: bool,
    /// `Type`: `x11`, `wayland`, `mir`, `tty`, `unspecified`.
    pub(super) kind: String,
    /// `IdleHint`: the desktop says the user is idle.
    pub(super) idle_hint: bool,
    /// `IdleSinceHint`: microseconds since the epoch; a tty's atime.
    pub(super) idle_since_us: u64,
}

/// The user idle time: the least idle local user session at `now_us`
/// (V5); `None` without one, a headless host (V2).
pub(super) fn idle(sessions: &[Session], now_us: u64) -> Option<Duration> {
    let _ = (sessions, now_us);
    None
}

/// The idle time logind reports over `stream`, authenticated as `uid`,
/// at `now_us` microseconds since the epoch.
pub(super) fn logind_idle<S: Read + Write>(
    stream: S,
    uid: u32,
    now_us: u64,
) -> Result<Option<Duration>, BusError> {
    let _ = (stream, uid, now_us);
    Err(BusError::Malformed("stub"))
}

#[cfg(test)]
mod tests;
