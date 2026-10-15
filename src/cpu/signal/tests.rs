use std::path::Path;
use std::time::Instant;

use super::{CpuSignal, Os, loads_from, macos_idle, run_queue_from};
use crate::cpu::{LoadWindow, Loads};
use crate::signal::{
    Counter, Measure, Reading, Root, Sample, Sampled, Signal, SignalId, Source,
};
use crate::sys::{CpuTicks, LoadAverage, PsetLoad};
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

#[test]
fn linux_run_queue_is_procs_running() {
    let sample = on("linux", Os::Linux).sample(SignalId::RunQueue);
    assert_eq!(sample, number(3_000));
}

#[test]
fn linux_run_queue_from_garbled_stat_is_unsupported() {
    assert_eq!(
        reason(on("garbled", Os::Linux).sample(SignalId::RunQueue)),
        Some("malformed /proc/stat line: `cpu 1 2 3`".to_owned())
    );
}

#[test]
fn linux_run_queue_past_64_bits_is_unsupported() {
    assert_eq!(
        reason(on("huge", Os::Linux).sample(SignalId::RunQueue)),
        Some("procs_running past 64 bits".to_owned())
    );
}

#[test]
fn macos_run_queue_is_a_number_from_the_processor_set() {
    let sample = on("absent", Os::Macos).sample(SignalId::RunQueue);
    let ok = match &sample {
        Sample::Gauge(Measure::Number(_)) => true,
        Sample::Unsupported(_) => !cfg!(target_os = "macos"),
        Sample::Gauge(_) | Sample::Counter(_) => false,
    };
    assert!(ok, "{sample:?}");
}

/// A processor set load of `load_average`.
fn pset(load_average: i32) -> std::io::Result<PsetLoad> {
    Ok(PsetLoad {
        load_average,
        ..PsetLoad::default()
    })
}

#[test]
fn the_pset_run_queue_is_its_load_in_thousandths() {
    assert_eq!(
        run_queue_from(pset(7_500)).ok(),
        Some(Decimal::from_thousandths(7_500))
    );
}

#[test]
fn a_negative_pset_load_is_unsupported() {
    let queue = run_queue_from(pset(-1)).map_err(|r| r.to_string());
    assert_eq!(
        queue,
        Err("processor_set_statistics: negative load -1".to_owned())
    );
}

#[test]
fn a_failed_pset_read_is_unsupported() {
    let err = std::io::Error::other("no mach");
    let queue = run_queue_from(Err(err)).map_err(|r| r.to_string());
    assert_eq!(queue, Err("no mach".to_owned()));
}

#[test]
fn linux_cpu_idle_counts_idle_and_iowait_of_all_ticks() {
    let sample = on("linux", Os::Linux).sample(SignalId::CpuIdle);
    let share = Counter::Share {
        part: 405,
        whole: 558,
    };
    assert_eq!(sample, Sample::Counter(share));
}

#[test]
fn linux_cpu_idle_past_64_bits_is_unsupported() {
    assert_eq!(
        reason(on("huge", Os::Linux).sample(SignalId::CpuIdle)),
        Some("/proc/stat cpu ticks past 64 bits".to_owned())
    );
}

#[test]
fn the_first_cpu_idle_reading_is_warming() {
    let mut source = Sampled::new(on("linux", Os::Linux));
    let reading = source.read(SignalId::CpuIdle, Instant::now());
    assert_eq!(reading, Reading::Warming);
}

#[test]
fn macos_cpu_idle_is_a_share_counter() {
    let sample = on("absent", Os::Macos).sample(SignalId::CpuIdle);
    let ok = match &sample {
        Sample::Counter(Counter::Share { .. }) => true,
        Sample::Unsupported(_) => !cfg!(target_os = "macos"),
        Sample::Gauge(_) | Sample::Counter(_) => false,
    };
    assert!(ok, "{sample:?}");
}

/// One CPU's ticks.
fn ticks(user: u64, system: u64, idle: u64, nice: u64) -> CpuTicks {
    CpuTicks {
        user,
        system,
        idle,
        nice,
    }
}

#[test]
fn macos_idle_sums_every_cpu() {
    let cpus = vec![ticks(10, 5, 80, 1), ticks(20, 10, 60, 0)];
    assert_eq!(
        macos_idle(Ok(cpus)).ok(),
        Some(Counter::Share {
            part: 140,
            whole: 186
        })
    );
}

#[test]
fn macos_idle_past_64_bits_is_unsupported() {
    let cpus = vec![ticks(0, 0, u64::MAX, 0), ticks(0, 0, 1, 0)];
    let idle = macos_idle(Ok(cpus)).map_err(|r| r.to_string());
    assert_eq!(
        idle,
        Err("host_processor_info ticks past 64 bits".to_owned())
    );
}

#[test]
fn a_failed_macos_ticks_read_is_unsupported() {
    let err = std::io::Error::other("no mach");
    let idle = macos_idle(Err(err)).map_err(|r| r.to_string());
    assert_eq!(idle, Err("no mach".to_owned()));
}
