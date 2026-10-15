use std::path::Path;

use super::{CpuSignal, Os, loads_from};
use crate::cpu::{LoadWindow, Loads};
use crate::signal::{Measure, Root, Sample, Signal, SignalId};
use crate::sys::LoadAverage;
use crate::units::Decimal;

/// The fixture tree `tests/fixtures/cpu/<case>`, laid out like `/`.
fn root(case: &str) -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/cpu").join(case))
}

/// A CPU signal over fixture `case` reading `os`'s sources.
fn on(case: &str, os: Os) -> CpuSignal {
    CpuSignal::on(root(case), os)
}

/// The reason of an unsupported sample, if it is one.
fn reason(sample: Sample) -> Option<String> {
    match sample {
        Sample::Unsupported(reason) => Some(reason.as_str().to_owned()),
        Sample::Gauge(_) | Sample::Counter(_) => None,
    }
}

/// A number gauge of `thousandths` thousandths.
const fn number(thousandths: u64) -> Sample {
    Sample::Gauge(Measure::Number(Decimal::from_thousandths(thousandths)))
}

#[test]
fn the_host_os_is_picked_by_new() {
    let want = if cfg!(target_os = "macos") {
        Os::Macos
    } else {
        Os::Linux
    };
    assert_eq!(CpuSignal::new(root("linux")).os, want);
}

#[test]
fn a_signal_of_another_domain_is_unsupported() {
    assert_eq!(
        reason(on("linux", Os::Linux).sample(SignalId::Mem)),
        Some("mem is not a cpu signal".to_owned())
    );
}

#[test]
fn linux_load_is_the_one_minute_average_of_loadavg() {
    let sample = on("linux", Os::Linux).sample(SignalId::Load);
    assert_eq!(sample, number(2_520));
}

#[test]
fn linux_load_from_garbled_loadavg_is_unsupported() {
    assert_eq!(
        reason(on("garbled", Os::Linux).sample(SignalId::Load)),
        Some("malformed /proc/loadavg line: `lots`".to_owned())
    );
}

#[test]
fn linux_load_without_loadavg_is_unsupported() {
    assert_eq!(
        reason(on("absent", Os::Linux).sample(SignalId::Load)),
        Some("/proc/loadavg: not present".to_owned())
    );
}

#[test]
fn macos_load_is_a_number_from_sysinfo() {
    let sample = on("absent", Os::Macos).sample(SignalId::Load);
    assert!(
        matches!(sample, Sample::Gauge(Measure::Number(_))),
        "{sample:?}"
    );
}

/// A `sysinfo` load of `one`, `five` and `fifteen`.
fn load(one: f64, five: f64, fifteen: f64) -> LoadAverage {
    LoadAverage { one, five, fifteen }
}

#[test]
fn sysinfo_loads_round_to_thousandths() {
    assert_eq!(
        loads_from(load(1.5, 0.12345, 7.0)).ok(),
        Some(Loads {
            one: Decimal::from_thousandths(1_500),
            five: Decimal::from_thousandths(123),
            fifteen: Decimal::from_thousandths(7_000),
        })
    );
}

#[test]
fn a_negative_sysinfo_load_is_unsupported() {
    let loads = loads_from(load(-1.0, 0.0, 0.0)).map_err(|r| r.to_string());
    assert_eq!(loads, Err("load average -1 is not a load".to_owned()));
}

#[test]
fn a_non_finite_sysinfo_load_is_unsupported() {
    let loads = loads_from(load(0.0, f64::NAN, 0.0)).map_err(|r| r.to_string());
    assert_eq!(loads, Err("load average NaN is not a load".to_owned()));
}

#[test]
fn linux_load_reads_the_window_asked_for() {
    let mut cpu = on("linux", Os::Linux).with_window(LoadWindow::Fifteen);
    assert_eq!(cpu.sample(SignalId::Load), number(750));
}
