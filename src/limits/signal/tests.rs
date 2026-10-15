use std::io;
use std::path::Path;

use super::{LimitsSignal, Os, sysctl_share};
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

/// Whether a live sample is a percent gauge on macOS, or anything but a
/// counter elsewhere: its value belongs to the machine, never asserted.
fn live_percent(sample: &Sample) -> bool {
    match sample {
        Sample::Gauge(Measure::Percent(_)) => true,
        Sample::Unsupported(_) => !cfg!(target_os = "macos"),
        Sample::Gauge(_) | Sample::Counter(_) => false,
    }
}

/// A share's reason as text.
fn text(
    share: Result<Percent, crate::signal::Reason>,
) -> Result<Percent, String> {
    share.map_err(|r| r.to_string())
}

#[test]
fn macos_fds_is_a_percent_from_sysctls() {
    let sample =
        LimitsSignal::on(root("absent"), Os::Macos).sample(SignalId::Fds);
    assert!(live_percent(&sample), "{sample:?}");
}

#[test]
fn sysctl_counts_make_a_share() {
    let share = sysctl_share(Ok(4096), Ok(16384), "kern.maxfiles");
    assert_eq!(share.ok(), Percent::from_hundredths(2500));
}

#[test]
fn a_sysctl_max_of_zero_is_unsupported() {
    let share = sysctl_share(Ok(1), Ok(0), "kern.maxfiles");
    assert_eq!(text(share), Err("kern.maxfiles is 0".to_owned()));
}

#[test]
fn a_negative_sysctl_count_is_unsupported() {
    let share = sysctl_share(Ok(-1), Ok(10), "kern.maxfiles");
    assert_eq!(text(share), Err("negative sysctl count -1".to_owned()));
}

#[test]
fn a_failed_sysctl_read_is_unsupported_with_its_error() {
    let err = io::Error::new(io::ErrorKind::Unsupported, "sysctl x: gone");
    let share = sysctl_share(Ok(1), Err(err), "kern.maxfiles");
    assert_eq!(text(share), Err("sysctl x: gone".to_owned()));
}
