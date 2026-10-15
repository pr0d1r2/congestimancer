//! Linux idle from logind: every session's idle facts over the system
//! bus, then the least idle local user session (src/presence:T2, V2, V5,
//! R1).

use std::io::{Read, Write};
use std::time::Duration;

use super::bus::Bus;
use super::error::BusError;
use super::wire::{Call, Value};

/// logind's bus name.
const LOGIND: &str = "org.freedesktop.login1";

/// logind's session interface, whose properties are read.
const SESSION: &str = "org.freedesktop.login1.Session";

/// logind's `ListSessions`: `a(susso)`, one row per session.
const LIST_SESSIONS: Call<'static> = Call {
    path: "/org/freedesktop/login1",
    interface: "org.freedesktop.login1.Manager",
    member: "ListSessions",
    destination: LOGIND,
    args: &[],
};

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

impl Session {
    /// A local user session (V5).
    fn counts(&self) -> bool {
        self.class == "user" && !self.remote
    }

    /// How long this session has been idle at `now_us` (R1): a desktop
    /// only when its hint says so, a tty since its atime.
    fn idle(&self, now_us: u64) -> Duration {
        if self.kind == "tty" || self.idle_hint {
            Duration::from_micros(now_us.saturating_sub(self.idle_since_us))
        } else {
            Duration::ZERO
        }
    }
}

/// The user idle time: the least idle local user session at `now_us`
/// (V5); `None` without one, a headless host (V2).
pub(super) fn idle(sessions: &[Session], now_us: u64) -> Option<Duration> {
    sessions
        .iter()
        .filter(|session| session.counts())
        .map(|session| session.idle(now_us))
        .min()
}

/// The idle time logind reports over `stream`, authenticated as `uid`,
/// at `now_us` microseconds since the epoch.
pub(super) fn logind_idle<S: Read + Write>(
    stream: S,
    uid: u32,
    now_us: u64,
) -> Result<Option<Duration>, BusError> {
    let mut bus = Bus::open(stream, uid)?;
    let sessions = sessions(&mut bus)?;
    Ok(idle(&sessions, now_us))
}

/// Every local user session logind lists; one that vanished between
/// the list and its properties is skipped.
fn sessions<S: Read + Write>(
    bus: &mut Bus<S>,
) -> Result<Vec<Session>, BusError> {
    let paths = bus.call(&LIST_SESSIONS)?.session_paths()?;
    let mut sessions = Vec::new();
    for path in paths {
        match session(bus, &path) {
            Ok(Some(session)) => sessions.push(session),
            Ok(None) | Err(BusError::Remote { .. }) => {}
            Err(err) => return Err(err),
        }
    }
    Ok(sessions)
}

/// The session at `path`, read only as far as it counts (V5).
fn session<S: Read + Write>(
    bus: &mut Bus<S>,
    path: &str,
) -> Result<Option<Session>, BusError> {
    let mut get = |name| property(bus, path, name);
    let class = text(get("Class")?)?;
    if class != "user" || flag(get("Remote")?)? {
        return Ok(None);
    }
    Ok(Some(Session {
        class,
        remote: false,
        kind: text(get("Type")?)?,
        idle_hint: flag(get("IdleHint")?)?,
        idle_since_us: number(get("IdleSinceHint")?)?,
    }))
}

/// Session property `name` of the session at `path`.
fn property<S: Read + Write>(
    bus: &mut Bus<S>,
    path: &str,
    name: &str,
) -> Result<Value, BusError> {
    let get = Call {
        path,
        interface: "org.freedesktop.DBus.Properties",
        member: "Get",
        destination: LOGIND,
        args: &[SESSION, name],
    };
    bus.call(&get)?.variant()
}

/// The property type logind documents did not come back.
const ODD_TYPE: BusError = BusError::Malformed("session property odd type");

/// An `s` property.
fn text(value: Value) -> Result<String, BusError> {
    match value {
        Value::Str(text) => Ok(text),
        Value::Bool(_) | Value::U64(_) => Err(ODD_TYPE),
    }
}

/// A `b` property.
fn flag(value: Value) -> Result<bool, BusError> {
    match value {
        Value::Bool(on) => Ok(on),
        Value::Str(_) | Value::U64(_) => Err(ODD_TYPE),
    }
}

/// A `t` property.
fn number(value: Value) -> Result<u64, BusError> {
    match value {
        Value::U64(n) => Ok(n),
        Value::Str(_) | Value::Bool(_) => Err(ODD_TYPE),
    }
}

#[cfg(test)]
mod tests;
