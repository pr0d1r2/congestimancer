use std::io;
use std::time::Duration;

use super::{Os, PresenceSignal, hid_idle, logind_reading};
use crate::presence::error::BusError;
use crate::signal::{Measure, Reason, Sample, Signal, SignalId};
use crate::units::ClockTime;

/// A wall clock tests set by hand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FixedWall {
    /// Microseconds since the Unix epoch.
    pub(super) micros: u64,
    /// The local time of day, or why there is none.
    pub(super) local: Result<ClockTime, Reason>,
}

impl super::WallClock for FixedWall {
    fn unix_micros(&self) -> u64 {
        self.micros
    }

    fn local_time(&self) -> Result<ClockTime, Reason> {
        self.local.clone()
    }
}

/// A wall clock at 1.7e15 microseconds and 10:30 local.
fn wall() -> FixedWall {
    FixedWall {
        micros: 1_700_000_000_000_000,
        local: ClockTime::new(10, 30).ok_or_else(|| Reason::from("bad")),
    }
}

/// The reason of an unsupported sample, if it is one.
fn reason(sample: Sample) -> Option<String> {
    match sample {
        Sample::Unsupported(reason) => Some(reason.as_str().to_owned()),
        Sample::Gauge(_) | Sample::Counter(_) => None,
    }
}

#[test]
fn the_host_os_is_picked_by_new() {
    let want = if cfg!(target_os = "macos") {
        Os::Macos
    } else {
        Os::Linux
    };
    assert_eq!(PresenceSignal::new().os, want);
    assert_eq!(PresenceSignal::default(), PresenceSignal::new());
}

#[test]
fn hid_idle_nanoseconds_are_the_idle_time() {
    assert_eq!(hid_idle(Ok(1_500_000_000)), Ok(Duration::from_millis(1500)));
    assert_eq!(hid_idle(Ok(0)), Ok(Duration::ZERO));
}

#[test]
fn no_hid_system_is_the_reason() {
    let err = io::Error::new(io::ErrorKind::Unsupported, "no IOHIDSystem");
    assert_eq!(hid_idle(Err(err)), Err(Reason::from("no IOHIDSystem")));
}

#[test]
fn a_signal_of_another_domain_is_unsupported() {
    let mut presence = PresenceSignal::on(Os::Macos, wall());
    assert_eq!(
        reason(presence.sample(SignalId::Load)),
        Some("load is not a presence signal".to_owned())
    );
}

#[test]
fn macos_idle_is_a_duration_or_unsupported() {
    // The value belongs to the machine; only its shape is asserted.
    let mut presence = PresenceSignal::on(Os::Macos, wall());
    assert!(matches!(
        presence.sample(SignalId::Idle),
        Sample::Gauge(Measure::Duration(_)) | Sample::Unsupported(_)
    ));
}

#[test]
fn logind_idle_is_the_idle_time() {
    let idle = Duration::from_secs(60);
    assert_eq!(logind_reading(Ok(Some(idle))), Ok(idle));
}

#[test]
fn no_local_session_is_headless() {
    assert_eq!(
        logind_reading(Ok(None)),
        Err(Reason::from("no local user session to read idle from"))
    );
}

#[test]
fn a_bus_failure_is_the_reason() {
    let err = BusError::Malformed("short");
    assert_eq!(logind_reading(Err(err)), Err(Reason::from("D-Bus: short")));
}

#[test]
fn linux_idle_is_a_duration_or_unsupported() {
    // Off Linux there is no bus; on it, the value is the machine's.
    let mut presence = PresenceSignal::on(Os::Linux, wall());
    assert!(matches!(
        presence.sample(SignalId::Idle),
        Sample::Gauge(Measure::Duration(_)) | Sample::Unsupported(_)
    ));
}
