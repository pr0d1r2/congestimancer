use std::io;
use std::path::Path;

use super::{
    BlockCounters, block_devices, bsd_name_of, decode_block_stats,
    mount_source, physical_disk_stats, walk_up,
};
use crate::sys::props::{Props, parse_props};

/// A property table captured on an M4 macOS 26 host.
fn captured(name: &str) -> Props {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir.join("tests/fixtures/sys/macos/iokit").join(name);
    let text = std::fs::read_to_string(path).unwrap_or_default();
    parse_props(&text).unwrap_or_default()
}

#[test]
fn statistics_decode_reads_and_writes() {
    let stats =
        decode_block_stats(&captured("IOBlockStorageDriver.Statistics"));
    let stats = stats.unwrap_or_default();
    let read = BlockCounters {
        bytes: 5_888_208_695_296,
        operations: 342_491_138,
        total_time_ns: 110_698_882_640_154,
        ..BlockCounters::default()
    };
    assert_eq!(stats.read, read);
    assert_eq!(stats.write.bytes, 3_198_276_878_336);
    assert_eq!(stats.write.operations, 203_402_954);
    assert_eq!(stats.write.total_time_ns, 22_451_268_585_861);
}

#[test]
fn optional_counters_are_read_when_present() {
    let mut table = captured("IOBlockStorageDriver.Statistics");
    for (key, n) in [("Errors (Read)", 2), ("Retries (Write)", 3)] {
        table.insert(key.to_owned(), crate::sys::props::PropValue::Int(n));
    }
    table.remove("Latency Time (Read)");
    let stats = decode_block_stats(&table).unwrap_or_default();
    assert_eq!((stats.read.errors, stats.write.retries), (2, 3));
    assert_eq!(stats.read.latency_time_ns, 0);
}

#[test]
fn a_table_without_required_counters_is_none() {
    let mut table = captured("IOBlockStorageDriver.Statistics");
    table.remove("Total Time (Write)");
    assert_eq!(decode_block_stats(&table), None);
    assert_eq!(decode_block_stats(&Props::new()), None);
}

#[test]
fn walking_up_stops_at_the_first_target() {
    let parent = |n: &u32| n.checked_sub(1);
    assert_eq!(walk_up(9, 32, parent, |n| n % 4 == 0), Some(8));
    assert_eq!(walk_up(8, 32, parent, |n| n % 4 == 0), Some(8));
}

#[test]
fn walking_up_gives_up_at_the_root_or_the_limit() {
    let parent = |n: &u32| n.checked_sub(1);
    assert_eq!(walk_up(3, 32, parent, |n| *n > 10), None);
    assert_eq!(walk_up(30, 5, parent, |n| *n == 0), None);
}

#[test]
fn a_device_path_becomes_its_bsd_name() {
    assert_eq!(bsd_name_of("/dev/disk3s1s1"), "disk3s1s1");
    assert_eq!(bsd_name_of("disk0"), "disk0");
}

#[cfg(target_os = "macos")]
#[test]
fn a_c_buffer_reads_up_to_its_nul() {
    let raw = [b'a', b'p', b'f', b's', 0, b'x'].map(|b| b as std::ffi::c_char);
    assert_eq!(super::imp::c_text(&raw), "apfs");
}

#[cfg(target_os = "macos")]
#[test]
fn an_unknown_device_is_unsupported() {
    let err = physical_disk_stats("/dev/cgmrnodisk9").err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::Unsupported));
    let err = physical_disk_stats("disk\0").err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::InvalidInput));
}

#[cfg(target_os = "macos")]
#[test]
fn a_missing_path_has_no_mount_source() {
    let err = mount_source(Path::new("/nonexistent/cgmr")).err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::NotFound));
}

#[cfg(not(target_os = "macos"))]
#[test]
fn other_oses_are_unsupported() {
    let unsupported = Some(io::ErrorKind::Unsupported);
    assert_eq!(
        mount_source(Path::new("/")).err().map(|e| e.kind()),
        unsupported
    );
    assert_eq!(
        physical_disk_stats("sda").err().map(|e| e.kind()),
        unsupported
    );
    assert_eq!(block_devices().err().map(|e| e.kind()), unsupported);
}

#[cfg(target_os = "macos")]
#[test]
fn host_root_volume_reaches_its_physical_disk() {
    let source = mount_source(Path::new("/")).ok();
    let device = source
        .as_ref()
        .map(|s| s.device.clone())
        .unwrap_or_default();
    assert_eq!(source.map(|s| s.fs_type), Some("apfs".to_owned()));
    let stats = physical_disk_stats(&device);
    assert!(stats.is_ok_and(|s| s.read.bytes > 0), "{device}");
    let disks = block_devices().unwrap_or_default();
    assert!(disks.iter().any(|d| d.name == "disk0"), "{disks:?}");
}
