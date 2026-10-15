use std::path::Path;

use super::{PowerState, decode_idle, decode_power, hid_idle_ns, power_state};
use crate::sys::props::{PropValue, Props, parse_props};

/// A property table captured on an M4 MacBook, macOS 26, from `iops`
/// (power sources) or `iokit` (registry entries).
fn captured(name: &str) -> Props {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir.join("tests/fixtures/sys/macos").join(name);
    let text = std::fs::read_to_string(path).unwrap_or_default();
    parse_props(&text).unwrap_or_default()
}

#[test]
fn ac_power_with_a_battery_decodes() {
    let battery = captured("iops/InternalBattery-0");
    let state = decode_power(Some("AC Power"), &[battery]);
    let want = PowerState {
        external: true,
        battery_percent: Some(80),
        charging: Some(false),
    };
    assert_eq!(state, want);
}

#[test]
fn battery_and_ups_power_are_not_external() {
    let battery = captured("iops/InternalBattery-0");
    let on_battery = decode_power(Some("Battery Power"), &[battery]);
    assert!(!on_battery.external);
    assert!(!decode_power(Some("UPS Power"), &[]).external);
}

#[test]
fn a_desktop_has_no_battery() {
    let state = decode_power(Some("AC Power"), &[]);
    let want = PowerState {
        external: true,
        battery_percent: None,
        charging: None,
    };
    assert_eq!(state, want);
}

#[test]
fn a_ups_is_not_the_internal_battery() {
    let mut ups = captured("iops/InternalBattery-0");
    ups.insert("Type".to_owned(), PropValue::Text("UPS".into()));
    let state = decode_power(Some("AC Power"), &[ups]);
    assert_eq!(state.battery_percent, None);
}

#[test]
fn charge_is_scaled_by_max_capacity() {
    let mut battery = captured("iops/InternalBattery-0");
    battery.insert("Current Capacity".to_owned(), PropValue::Int(3000));
    battery.insert("Max Capacity".to_owned(), PropValue::Int(6000));
    let state = decode_power(None, &[battery.clone()]);
    assert_eq!(state.battery_percent, Some(50));
    battery.insert("Max Capacity".to_owned(), PropValue::Int(0));
    assert_eq!(decode_power(None, &[battery]).battery_percent, None);
}

#[test]
fn hid_idle_time_decodes() {
    let props = captured("iokit/IOHIDSystem");
    assert_eq!(decode_idle(&props), Some(7_365_740_312_291));
    assert_eq!(decode_idle(&Props::new()), None);
}

#[cfg(not(target_os = "macos"))]
#[test]
fn other_oses_are_unsupported() {
    use std::io::ErrorKind::Unsupported;

    assert_eq!(power_state().err().map(|e| e.kind()), Some(Unsupported));
    assert_eq!(hid_idle_ns().err().map(|e| e.kind()), Some(Unsupported));
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "live machine (src:V4): run with --ignored --nocapture"]
fn live_power_and_idle_read() {
    let (providing, sources) = super::imp::power_sources().unwrap_or_default();
    assert!(providing.is_some());
    for source in &sources {
        println!("{}", crate::sys::props::render_props(source));
    }
    assert!(power_state().is_ok());
    assert!(hid_idle_ns().is_ok());
}
