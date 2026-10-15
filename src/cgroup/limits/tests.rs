use std::path::Path;
use std::time::Duration;

use super::{
    CpuMax, memory_available, parse_cpu_max, parse_memory_current,
    parse_memory_max,
};
use crate::cgroup::CgroupError;
use crate::units::Bytes;

/// The text of a file under `tests/fixtures/cgroup/limited`'s job cgroup.
fn job(name: &str) -> String {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cgroup/limited/sys/fs/cgroup/user.slice");
    std::fs::read_to_string(dir.join("job").join(name)).unwrap_or_default()
}

#[test]
fn a_quota_limits_the_cpu() {
    let max = parse_cpu_max(&job("cpu.max"));
    let micros = Duration::from_micros;
    assert_eq!(
        max,
        Ok(CpuMax {
            quota: Some(micros(200_000)),
            period: micros(100_000),
        })
    );
    assert!(max.is_ok_and(|m| m.is_limited()));
}

#[test]
fn max_quota_is_no_limit() {
    let max = parse_cpu_max("max 100000\n");
    assert_eq!(max.map(|m| (m.quota, m.is_limited())), Ok((None, false)));
}

#[test]
fn malformed_cpu_max_is_an_error() {
    for text in ["", "max", "1 2 3", "x 100000", "100 y", "100 0"] {
        let parsed = parse_cpu_max(text);
        assert!(
            matches!(parsed, Err(CgroupError::Malformed { .. })),
            "{text:?} parsed as {parsed:?}"
        );
    }
}

#[test]
fn memory_max_is_bytes_or_no_limit() {
    assert_eq!(parse_memory_max("8589934592\n"), Ok(Some(Bytes(1 << 33))));
    assert_eq!(parse_memory_max(&job("memory.max")), Ok(None));
    let bad = parse_memory_max("lots\n");
    assert!(matches!(bad, Err(CgroupError::Malformed { .. })));
}

#[test]
fn memory_current_is_bytes() {
    let current = parse_memory_current(&job("memory.current"));
    assert_eq!(current, Ok(Bytes(1 << 30)));
    let bad = parse_memory_current("max\n");
    assert!(matches!(bad, Err(CgroupError::Malformed { .. })));
}

#[test]
fn available_is_the_limit_left_capped_by_the_host() {
    let gib = |n: u64| Bytes(n << 30);
    assert_eq!(memory_available(gib(8), gib(1), None), gib(7));
    assert_eq!(memory_available(gib(8), gib(1), Some(gib(2))), gib(2));
    assert_eq!(memory_available(gib(8), gib(1), Some(gib(9))), gib(7));
}

#[test]
fn use_past_the_limit_leaves_nothing() {
    let gib = |n: u64| Bytes(n << 30);
    assert_eq!(memory_available(gib(1), gib(2), Some(gib(4))), Bytes(0));
}
