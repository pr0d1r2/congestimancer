use std::path::Path;

use super::{
    is_cpu_cooling, parse_cpu_list, parse_millicelsius, parse_sysfs_u64,
};
use crate::power::PowerError;
use crate::units::Celsius;

/// The text of fixture `tests/fixtures/power/<path>`.
fn fixture(path: &str) -> String {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(dir.join("tests/fixtures/power").join(path))
        .unwrap_or_default()
}

#[test]
fn a_zone_temperature_is_millidegrees_in_tenths() {
    let text = fixture("intel/sys/class/thermal/thermal_zone1/temp");
    assert_eq!(parse_millicelsius(&text), Ok(Celsius::from_tenths(615)));
}

#[test]
fn a_zone_temperature_rounds_down_to_tenths() {
    assert_eq!(parse_millicelsius("45099\n"), Ok(Celsius::from_tenths(450)));
}

#[test]
fn a_zone_below_zero_reads_zero() {
    let text = fixture("arm/sys/class/thermal/thermal_zone1/temp");
    assert_eq!(parse_millicelsius(&text), Ok(Celsius::from_tenths(0)));
}

#[test]
fn a_zone_temperature_that_is_not_a_number_is_malformed() {
    assert_eq!(
        parse_millicelsius("hot\n"),
        Err(PowerError::malformed("temp", "hot"))
    );
}

#[test]
fn a_throttle_counter_is_a_whole_number() {
    let path = "intel/sys/devices/system/cpu/cpu0/thermal_throttle/\
                core_throttle_count";
    let text = fixture(path);
    assert_eq!(parse_sysfs_u64("core_throttle_count", &text), Ok(12));
}

#[test]
fn a_negative_counter_is_malformed_naming_the_file() {
    assert_eq!(
        parse_sysfs_u64("cur_state", "-1"),
        Err(PowerError::malformed("cur_state", "-1"))
    );
}

#[test]
fn a_cpu_list_expands_ranges_and_singles() {
    assert_eq!(parse_cpu_list("0-2,5,7-8\n"), Ok(vec![0, 1, 2, 5, 7, 8]));
}

#[test]
fn the_online_fixture_lists_two_cpus() {
    let text = fixture("intel/sys/devices/system/cpu/online");
    assert_eq!(parse_cpu_list(&text), Ok(vec![0, 1]));
}

#[test]
fn an_empty_cpu_list_is_empty() {
    assert_eq!(parse_cpu_list("\n"), Ok(Vec::new()));
}

#[test]
fn a_backwards_range_is_malformed() {
    assert_eq!(
        parse_cpu_list("3-1"),
        Err(PowerError::malformed("cpu list", "3-1"))
    );
}

#[test]
fn a_part_that_is_not_a_number_is_malformed() {
    assert_eq!(
        parse_cpu_list("0-x"),
        Err(PowerError::malformed("cpu list", "0-x"))
    );
}

#[test]
fn a_list_past_the_cpu_cap_is_malformed() {
    assert_eq!(
        parse_cpu_list("0-4294967295"),
        Err(PowerError::malformed("cpu list", "0-4294967295"))
    );
}

#[test]
fn processor_and_cpufreq_devices_cool_the_cpu() {
    assert!(is_cpu_cooling("Processor\n"));
    assert!(is_cpu_cooling("cpufreq-cpu0"));
    assert!(is_cpu_cooling("thermal-cpufreq-0"));
}

#[test]
fn a_fan_does_not_cool_the_cpu_by_throttling() {
    assert!(!is_cpu_cooling("Fan"));
    assert!(!is_cpu_cooling("intel_powerclamp"));
}
