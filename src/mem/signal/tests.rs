use std::path::Path;

use super::{MemSignal, Os, level_from};
use crate::signal::{Counter, Level, Measure, Root, Sample, Signal, SignalId};
use crate::units::Percent;

/// The fixture tree `tests/fixtures/mem/<case>`, laid out like `/`.
fn root(case: &str) -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/mem").join(case))
}

/// A mem signal over fixture `case` reading `os`'s sources.
fn on(case: &str, os: Os) -> MemSignal {
    MemSignal::on(root(case), os)
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
    assert_eq!(MemSignal::new(root("linux")).os, want);
}

#[test]
fn a_signal_of_another_domain_is_unsupported() {
    assert_eq!(
        reason(on("linux", Os::Linux).sample(SignalId::Load)),
        Some("load is not a mem signal".to_owned())
    );
}

/// Whether `sample` is a bytes gauge: live memory belongs to the
/// machine, so its value is never asserted.
fn is_bytes(sample: &Sample) -> bool {
    matches!(sample, Sample::Gauge(Measure::Bytes(_)))
}

#[test]
fn mem_is_available_bytes_on_linux_sources() {
    let sample = on("linux", Os::Linux).sample(SignalId::Mem);
    assert!(is_bytes(&sample), "{sample:?}");
}

#[test]
fn mem_is_available_bytes_on_macos_sources() {
    let sample = on("linux", Os::Macos).sample(SignalId::Mem);
    assert!(is_bytes(&sample), "{sample:?}");
}

/// A percent gauge of `hundredths` hundredths.
fn percent(hundredths: u16) -> Option<Sample> {
    Percent::from_hundredths(hundredths)
        .map(|p| Sample::Gauge(Measure::Percent(p)))
}

#[test]
fn linux_mem_pressure_is_some_avg10() {
    let sample = on("linux", Os::Linux).sample(SignalId::MemPressure);
    assert_eq!(Some(sample), percent(153));
}

#[test]
fn linux_mem_pressure_full_is_full_avg10() {
    let sample = on("linux", Os::Linux).sample(SignalId::MemPressureFull);
    assert_eq!(Some(sample), percent(42));
}

#[test]
fn linux_mem_pressure_full_without_a_full_line_is_unsupported() {
    let sample = on("partial", Os::Linux).sample(SignalId::MemPressureFull);
    assert_eq!(
        reason(sample),
        Some("/proc/pressure/memory has no `full` line".to_owned())
    );
}

#[test]
fn linux_mem_pressure_from_garbled_psi_is_unsupported() {
    let sample = on("garbled", Os::Linux).sample(SignalId::MemPressure);
    assert_eq!(
        reason(sample),
        Some("malformed PSI line: some avg10=lots".to_owned())
    );
}

#[test]
fn linux_mem_pressure_without_psi_is_unsupported() {
    let sample = on("absent", Os::Linux).sample(SignalId::MemPressure);
    assert_eq!(
        reason(sample),
        Some("/proc/pressure/memory: not present".to_owned())
    );
}

#[test]
fn linux_mem_level_is_unsupported() {
    assert_eq!(
        reason(on("linux", Os::Linux).sample(SignalId::MemLevel)),
        Some(
            "no memory pressure level on Linux; use --mem-pressure".to_owned()
        )
    );
}

#[test]
fn macos_mem_level_is_a_level_from_the_kernel() {
    let sample = on("linux", Os::Macos).sample(SignalId::MemLevel);
    let ok = match &sample {
        Sample::Gauge(Measure::Level(_)) => true,
        Sample::Unsupported(_) => !cfg!(target_os = "macos"),
        Sample::Gauge(_) | Sample::Counter(_) => false,
    };
    assert!(ok, "{sample:?}");
}

#[test]
fn macos_mem_pressure_is_unsupported() {
    for id in [SignalId::MemPressure, SignalId::MemPressureFull] {
        assert_eq!(
            reason(on("linux", Os::Macos).sample(id)),
            Some("no PSI on macOS; use --mem-level".to_owned())
        );
    }
}

#[test]
fn a_known_level_read_decodes() {
    assert_eq!(level_from(Ok(2)).ok(), Some(Level::Warning));
}

#[test]
fn an_unknown_level_read_is_unsupported() {
    let level = level_from(Ok(3)).map_err(|r| r.to_string());
    assert_eq!(
        level,
        Err("kern.memorystatus_vm_pressure_level: unknown level 3".to_owned())
    );
}

#[test]
fn a_failed_level_read_is_unsupported() {
    let err = std::io::Error::other("no sysctl");
    let level = level_from(Err(err)).map_err(|r| r.to_string());
    assert_eq!(level, Err("no sysctl".to_owned()));
}

#[test]
fn linux_swap_rate_counts_swapped_pages_in_bytes() {
    let sample = on("linux", Os::Linux).sample(SignalId::SwapRate);
    let total = (1200 + 3400) * 4096;
    assert_eq!(sample, Sample::Counter(Counter::Rate { total }));
}

#[test]
fn linux_swap_rate_from_garbled_vmstat_is_unsupported() {
    assert_eq!(
        reason(on("garbled", Os::Linux).sample(SignalId::SwapRate)),
        Some("malformed /proc/vmstat line: `pswpin lots`".to_owned())
    );
}

#[test]
fn linux_swap_rate_without_a_page_size_is_unsupported() {
    assert_eq!(
        reason(on("partial", Os::Linux).sample(SignalId::SwapRate)),
        Some("/proc/self/smaps has no `KernelPageSize`".to_owned())
    );
}

#[test]
fn linux_swap_rate_without_vmstat_is_unsupported() {
    assert_eq!(
        reason(on("absent", Os::Linux).sample(SignalId::SwapRate)),
        Some("/proc/vmstat: not present".to_owned())
    );
}
