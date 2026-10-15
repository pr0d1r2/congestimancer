use std::path::Path;

use super::{LimitsSignal, Os};
use crate::signal::{Measure, Root, Sample, Signal, SignalId};
use crate::units::Percent;

/// The fixture tree `tests/fixtures/limits/<case>`, laid out like `/`.
fn root(case: &str) -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/limits").join(case))
}

/// A Linux limits signal over fixture `case`.
fn linux(case: &str) -> LimitsSignal {
    LimitsSignal::on(root(case), Os::Linux)
}

/// The reason of an unsupported sample, if it is one.
fn reason(sample: Sample) -> Option<String> {
    match sample {
        Sample::Unsupported(reason) => Some(reason.as_str().to_owned()),
        Sample::Gauge(_) | Sample::Counter(_) => None,
    }
}

/// A percent gauge of `hundredths` hundredths.
fn percent(hundredths: u16) -> Option<Sample> {
    Percent::from_hundredths(hundredths)
        .map(|p| Sample::Gauge(Measure::Percent(p)))
}

#[test]
fn the_host_os_is_picked_by_new() {
    let want = if cfg!(target_os = "macos") {
        Os::Macos
    } else {
        Os::Linux
    };
    assert_eq!(LimitsSignal::new(root("linux")).os, want);
}

#[test]
fn a_signal_of_another_domain_is_unsupported() {
    assert_eq!(
        reason(linux("linux").sample(SignalId::Load)),
        Some("load is not a limits signal".to_owned())
    );
}

#[test]
fn linux_fds_is_open_files_over_file_max() {
    assert_eq!(Some(linux("linux").sample(SignalId::Fds)), percent(2500));
}

#[test]
fn linux_fds_with_a_file_max_of_zero_is_unsupported() {
    assert_eq!(
        reason(linux("zero").sample(SignalId::Fds)),
        Some("fs.file-max is 0".to_owned())
    );
}

#[test]
fn linux_fds_with_a_malformed_file_nr_is_unsupported() {
    assert_eq!(
        reason(linux("garbled").sample(SignalId::Fds)),
        Some("malformed file-nr: `lots`".to_owned())
    );
}

#[test]
fn linux_fds_without_file_nr_is_unsupported() {
    assert_eq!(
        reason(linux("absent").sample(SignalId::Fds)),
        Some("/proc/sys/fs/file-nr: not present".to_owned())
    );
}
