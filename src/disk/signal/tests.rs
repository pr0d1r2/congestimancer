use std::path::{Path, PathBuf};

use super::{DiskSignal, free_space};
use crate::signal::{Measure, Sample, Signal, SignalId};
use crate::sys::FsStats;
use crate::units::Bytes;

/// The fixture directory `tests/fixtures/disk/<case>`.
fn fixture(case: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/disk")
        .join(case)
}

/// Stats of 4 KiB fragments: 1000 blocks, 300 free, 200 for users.
fn stats() -> FsStats {
    FsStats {
        fragment_size: 4096,
        blocks: 1000,
        blocks_free: 300,
        blocks_available: 200,
        ..FsStats::default()
    }
}

/// The reason of an unsupported sample, if it is one.
fn reason(sample: Sample) -> Option<String> {
    match sample {
        Sample::Unsupported(reason) => Some(reason.as_str().to_owned()),
        Sample::Gauge(_) | Sample::Counter(_) => None,
    }
}

#[test]
fn free_space_is_blocks_available_times_fragment_size() {
    assert_eq!(free_space(&stats()), Bytes(200 * 4096));
}

#[test]
fn free_space_saturates() {
    let mut stats = stats();
    stats.blocks_available = u64::MAX;
    assert_eq!(free_space(&stats), Bytes(u64::MAX));
}

#[test]
fn disk_reads_bytes_of_the_filesystem_holding_the_path() {
    let mut disk = DiskSignal::new(&fixture("mount"));
    assert!(matches!(
        disk.sample(SignalId::Disk),
        Sample::Gauge(Measure::Bytes(_))
    ));
}

#[test]
fn disk_on_a_path_that_is_gone_is_unsupported() {
    let gone = fixture("gone");
    let mut disk = DiskSignal::new(&gone);
    let why = reason(disk.sample(SignalId::Disk)).unwrap_or_default();
    assert!(why.starts_with(&gone.display().to_string()), "{why}");
}

#[test]
fn a_signal_of_another_domain_is_unsupported() {
    let mut disk = DiskSignal::new(&fixture("mount"));
    assert_eq!(
        reason(disk.sample(SignalId::Load)),
        Some("load is not a disk signal".to_owned())
    );
}
