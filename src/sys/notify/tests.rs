use std::io;

use super::{THERMAL_PRESSURE, notify_state, status_error};

#[test]
fn a_status_names_the_call_and_the_notification() {
    let err = status_error("notify_get_state", "com.example.x", 2);
    assert_eq!(err.kind(), io::ErrorKind::Other);
    assert_eq!(
        err.to_string(),
        "notify_get_state com.example.x: notify status 2"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn a_name_never_posted_reads_zero() {
    let name = format!("org.congestimancer.test.{}", std::process::id());
    assert_eq!(notify_state(&name).ok(), Some(0));
}

#[cfg(target_os = "macos")]
#[test]
fn a_name_with_a_nul_is_invalid_input() {
    let err = notify_state("com.apple\0x").err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::InvalidInput));
}

#[cfg(not(target_os = "macos"))]
#[test]
fn other_oses_are_unsupported() {
    let err = notify_state(THERMAL_PRESSURE).err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::Unsupported));
}

#[cfg(target_os = "macos")]
#[test]
fn host_thermal_and_low_power_read() {
    assert!(notify_state(THERMAL_PRESSURE).is_ok());
    assert!(notify_state(super::LOW_POWER_MODE).is_ok());
}
