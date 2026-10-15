use std::path::Path;

use super::{
    Supply, SupplyKind, battery_charge, on_mains, parse_supply_uevent,
    supplies_of,
};
use crate::power::PowerError;
use crate::sys::PowerState;
use crate::units::Percent;

/// The supply `name` of fixture tree `case`, parsed.
fn supply(case: &str, name: &str) -> Supply {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir
        .join("tests/fixtures/power")
        .join(case)
        .join("sys/class/power_supply")
        .join(name)
        .join("uevent");
    let text = std::fs::read_to_string(path).unwrap_or_default();
    parse_supply_uevent(&text).unwrap_or(BARE)
}

/// A supply nothing is known about, for failed fixtures.
const BARE: Supply = Supply {
    kind: SupplyKind::Other,
    online: None,
    present: false,
    capacity: None,
    device: false,
};

/// `whole` percent.
fn percent(whole: u16) -> Option<Percent> {
    Percent::from_hundredths(whole.saturating_mul(100))
}

#[test]
fn an_online_adapter_is_mains_online() {
    let ac = supply("intel", "AC");
    assert_eq!(ac.kind, SupplyKind::Mains);
    assert_eq!(ac.online, Some(true));
    assert!(ac.present);
    assert!(!ac.is_system_battery());
}

#[test]
fn a_charging_battery_reads_its_capacity() {
    let battery = supply("intel", "BAT0");
    assert_eq!(battery.kind, SupplyKind::Battery);
    assert_eq!(battery.capacity, percent(80));
    assert!(battery.is_system_battery());
}

#[test]
fn a_mouse_battery_is_not_the_machines() {
    let mouse = supply("intel", "hidpp_battery_0");
    assert!(mouse.device);
    assert!(!mouse.is_system_battery());
}

#[test]
fn an_empty_bay_is_not_a_battery() {
    let bay = supply("arm", "BAT0");
    assert!(!bay.present);
    assert!(!bay.is_system_battery());
}

#[test]
fn capacity_falls_back_to_energy_now_of_full() {
    assert_eq!(supply("arm", "BAT1").capacity, percent(75));
}

#[test]
fn capacity_falls_back_to_charge_now_of_full() {
    let text = "POWER_SUPPLY_TYPE=Battery\nPOWER_SUPPLY_CHARGE_NOW=500\n\
                POWER_SUPPLY_CHARGE_FULL=2000\n";
    assert_eq!(
        parse_supply_uevent(text).map(|s| s.capacity),
        Ok(percent(25))
    );
}

#[test]
fn capacity_past_full_reads_full() {
    let text = "POWER_SUPPLY_TYPE=Battery\nPOWER_SUPPLY_CAPACITY=104\n";
    assert_eq!(
        parse_supply_uevent(text).map(|s| s.capacity),
        Ok(percent(100))
    );
}

#[test]
fn usb_ups_and_unknown_types_are_named() {
    let kind = |t: &str| {
        let text = format!("POWER_SUPPLY_TYPE={t}\n");
        parse_supply_uevent(&text).map(|s| s.kind)
    };
    assert_eq!(kind("USB"), Ok(SupplyKind::Usb));
    assert_eq!(kind("UPS"), Ok(SupplyKind::Ups));
    assert_eq!(kind("Wireless"), Ok(SupplyKind::Other));
}

#[test]
fn a_uevent_without_a_type_is_malformed() {
    assert_eq!(
        parse_supply_uevent("POWER_SUPPLY_ONLINE=1\n"),
        Err(PowerError::malformed("uevent", "POWER_SUPPLY_ONLINE=1"))
    );
}

#[test]
fn an_online_adapter_is_mains_power() {
    let supplies = [supply("intel", "AC"), supply("intel", "BAT0")];
    assert!(on_mains(&supplies));
}

#[test]
fn an_offline_adapter_with_a_battery_is_not_mains_power() {
    let supplies = [supply("arm", "ADP1"), supply("arm", "BAT1")];
    assert!(!on_mains(&supplies));
}

#[test]
fn a_desktop_without_supplies_is_on_mains_power() {
    assert!(on_mains(&[]));
}

#[test]
fn only_a_mouse_battery_is_still_a_desktop() {
    assert!(on_mains(&[supply("intel", "hidpp_battery_0")]));
}

#[test]
fn charge_is_the_mean_of_the_machines_batteries() {
    let supplies = [
        supply("intel", "BAT0"),
        supply("arm", "BAT1"),
        supply("intel", "hidpp_battery_0"),
        supply("intel", "AC"),
    ];
    assert_eq!(battery_charge(&supplies), Percent::from_hundredths(7750));
}

#[test]
fn no_battery_has_no_charge() {
    assert_eq!(battery_charge(&[supply("intel", "AC")]), None);
}

#[test]
fn macos_wall_power_with_a_battery_is_two_supplies() {
    let state = PowerState {
        external: true,
        battery_percent: Some(42),
        charging: Some(true),
    };
    let supplies = supplies_of(&state);
    assert!(on_mains(&supplies));
    assert_eq!(battery_charge(&supplies), percent(42));
}

#[test]
fn macos_on_battery_is_not_mains_power() {
    let state = PowerState {
        external: false,
        battery_percent: Some(30),
        charging: Some(false),
    };
    assert!(!on_mains(&supplies_of(&state)));
}

#[test]
fn a_mac_desktop_has_no_battery_charge() {
    let state = PowerState {
        external: true,
        battery_percent: None,
        charging: None,
    };
    let supplies = supplies_of(&state);
    assert!(on_mains(&supplies));
    assert_eq!(battery_charge(&supplies), None);
}
