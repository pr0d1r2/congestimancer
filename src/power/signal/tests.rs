use std::path::Path;

use super::{Os, PowerSignal};
use crate::signal::{Measure, Root, Sample, Signal, SignalId};

/// The fixture tree `tests/fixtures/power/<case>`, laid out like `/`.
fn root(case: &str) -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/power").join(case))
}

/// A Linux power signal over fixture `case`.
fn linux(case: &str) -> PowerSignal {
    PowerSignal::on(root(case), Os::Linux)
}

/// The reason of an unsupported sample, if it is one.
fn reason(sample: Sample) -> Option<String> {
    match sample {
        Sample::Unsupported(reason) => Some(reason.as_str().to_owned()),
        Sample::Gauge(_) | Sample::Counter(_) => None,
    }
}

/// A yes or no gauge.
const fn flag(on: bool) -> Sample {
    Sample::Gauge(Measure::Flag(on))
}

/// Whether a live macOS sample is a gauge of `want`'s kind or
/// unsupported: its value belongs to the machine, never asserted.
fn shaped(sample: &Sample, want: fn(&Measure) -> bool) -> bool {
    match sample {
        Sample::Gauge(measure) => want(measure),
        Sample::Unsupported(_) => true,
        Sample::Counter(_) => false,
    }
}

#[test]
fn the_host_os_is_picked_by_new() {
    let want = if cfg!(target_os = "macos") {
        Os::Macos
    } else {
        Os::Linux
    };
    assert_eq!(PowerSignal::new(root("intel")).os, want);
}

#[test]
fn a_signal_of_another_domain_is_unsupported() {
    assert_eq!(
        reason(linux("intel").sample(SignalId::Load)),
        Some("load is not a power signal".to_owned())
    );
}

#[test]
fn first_cool_sample_asks_the_idle_cooling_device() {
    assert_eq!(linux("intel").sample(SignalId::Cool), flag(true));
}

#[test]
fn unchanged_throttle_counters_are_cool() {
    let mut power = linux("intel");
    power.sample(SignalId::Cool);
    assert_eq!(power.sample(SignalId::Cool), flag(true));
}

#[test]
fn grown_throttle_counters_are_not_cool() {
    let mut power = linux("intel");
    power.sample(SignalId::Cool);
    power.root = root("intel-hot");
    assert_eq!(power.sample(SignalId::Cool), flag(false));
}

#[test]
fn counters_without_a_cooling_device_start_cool() {
    let mut power = linux("intel-hot");
    assert_eq!(power.sample(SignalId::Cool), flag(true));
    assert_eq!(power.sample(SignalId::Cool), flag(true));
}

#[test]
fn an_active_cpufreq_cooling_device_is_not_cool() {
    let mut power = linux("arm");
    assert_eq!(power.sample(SignalId::Cool), flag(false));
    assert_eq!(power.sample(SignalId::Cool), flag(false));
}

#[test]
fn a_busy_fan_with_an_idle_processor_device_is_cool() {
    assert_eq!(linux("amd").sample(SignalId::Cool), flag(true));
}

#[test]
fn no_counters_and_no_cooling_devices_is_unsupported() {
    assert_eq!(
        reason(linux("none").sample(SignalId::Cool)),
        Some("no thermal_throttle counters or CPU cooling devices".to_owned())
    );
}

#[test]
fn macos_cool_is_a_flag_from_thermal_pressure() {
    let sample =
        PowerSignal::on(root("none"), Os::Macos).sample(SignalId::Cool);
    assert!(shaped(&sample, |m| matches!(m, Measure::Flag(_))));
}
