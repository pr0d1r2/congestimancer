use std::path::Path;

use super::{Direction, DiskStat, direction, parse_diskstats};
use crate::io::IoError;

/// The text of fixture `tests/fixtures/io/<case>/proc/diskstats`.
fn fixture(case: &str) -> String {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/io");
    let path = dir.join(case).join("proc/diskstats");
    std::fs::read_to_string(path).unwrap_or_default()
}

/// The devices of fixture `case`.
fn devices(case: &str) -> Vec<DiskStat> {
    parse_diskstats(&fixture(case)).unwrap_or_default()
}

#[test]
fn every_device_line_is_one_stat() {
    let names: Vec<String> =
        devices("linux").into_iter().map(|d| d.name).collect();
    assert_eq!(names, ["nvme0n1", "nvme0n1p1", "sda", "dm-0"]);
}

#[test]
fn a_line_reads_its_counters_by_column() {
    let first = devices("linux").into_iter().next();
    let want = DiskStat {
        major: 259,
        minor: 0,
        name: "nvme0n1".to_owned(),
        read: direction(120, 4000, 300),
        write: direction(80, 2000, 500),
        in_flight: 2,
        io_ticks_ms: 700,
    };
    assert_eq!(first, Some(want));
}

#[test]
fn an_old_kernel_line_without_discards_parses() {
    let sda = devices("linux").into_iter().find(|d| d.name == "sda");
    assert_eq!(sda.map(|d| (d.in_flight, d.io_ticks_ms)), Some((0, 15)));
}

#[test]
fn sectors_are_512_bytes() {
    assert_eq!(direction(1, 3, 0).bytes(), Some(1536));
    assert_eq!(direction(1, u64::MAX, 0).bytes(), None);
}

#[test]
fn a_short_or_garbled_line_is_malformed() {
    assert_eq!(
        parse_diskstats(&fixture("garbled")),
        Err(IoError::malformed("/proc/diskstats", "8 0 sda 1 2 three"))
    );
}

#[test]
fn blank_text_is_no_devices() {
    assert_eq!(parse_diskstats("\n  \n"), Ok(Vec::new()));
}

#[test]
fn a_direction_defaults_to_zero() {
    assert_eq!(Direction::default(), direction(0, 0, 0));
}
