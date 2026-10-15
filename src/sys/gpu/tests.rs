use std::path::Path;

use super::{decode_usage, gpu_usage};
use crate::sys::props::{PropValue, Props, parse_props};

/// `PerformanceStatistics` captured on an M4 macOS 26 host.
fn captured() -> Props {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir.join(
        "tests/fixtures/sys/macos/iokit/IOAccelerator.PerformanceStatistics",
    );
    let text = std::fs::read_to_string(path).unwrap_or_default();
    parse_props(&text).unwrap_or_default()
}

#[test]
fn apple_silicon_reports_utilization_without_vram() {
    let usage = decode_usage(captured());
    assert_eq!(usage.utilization_percent, Some(20));
    assert_eq!(usage.vram_free_bytes, None);
    assert_eq!(
        usage.stats.get("Alloc system memory"),
        Some(&PropValue::Int(6_518_194_176))
    );
}

#[test]
fn a_discrete_gpu_reports_free_vram() {
    let mut stats = captured();
    stats.insert("vramFreeBytes".to_owned(), PropValue::Int(1 << 30));
    assert_eq!(decode_usage(stats).vram_free_bytes, Some(1 << 30));
}

#[test]
fn a_table_without_utilization_has_none() {
    let usage = decode_usage(Props::new());
    assert_eq!(usage.utilization_percent, None);
}

#[cfg(not(target_os = "macos"))]
#[test]
fn other_oses_are_unsupported() {
    let err = gpu_usage().err().map(|e| e.kind());
    assert_eq!(err, Some(std::io::ErrorKind::Unsupported));
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "live machine (src:V4): run with --ignored"]
fn live_gpu_usage_reads() {
    let usage = gpu_usage().unwrap_or_default();
    assert!(usage.iter().any(|u| u.utilization_percent.is_some()));
}
