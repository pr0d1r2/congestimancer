use std::path::Path;
use std::time::Duration;

use super::{CpuStat, Throttling, parse_cpu_stat, parse_unreclaimable};
use crate::cgroup::CgroupError;
use crate::units::Bytes;

/// The text of the job cgroup's `cpu.stat` in fixture `limited`.
fn fixture() -> String {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let file = "tests/fixtures/cgroup/limited/sys/fs/cgroup/user.slice/job";
    std::fs::read_to_string(dir.join(file).join("cpu.stat")).unwrap_or_default()
}

#[test]
fn reads_usage_and_throttling() {
    let micros = Duration::from_micros;
    assert_eq!(
        parse_cpu_stat(&fixture()),
        Ok(CpuStat {
            usage: micros(9_876_543),
            throttling: Some(Throttling {
                periods: 420,
                throttled_periods: 17,
                throttled: micros(1_234_567),
            }),
        })
    );
}

#[test]
fn no_cpu_controller_means_no_throttling() {
    let text = "usage_usec 10\nuser_usec 6\nsystem_usec 4\n";
    let stat = parse_cpu_stat(text);
    assert_eq!(stat.map(|s| s.throttling), Ok(None));
}

#[test]
fn usage_is_required() {
    assert_eq!(
        parse_cpu_stat("nr_periods 1\n"),
        Err(CgroupError::Missing {
            file: "cpu.stat",
            key: "usage_usec",
        })
    );
}

#[test]
fn malformed_lines_are_errors() {
    for text in ["usage_usec\n", "usage_usec x\n", "usage_usec 1 2\n"] {
        let parsed = parse_cpu_stat(text);
        assert!(
            matches!(parsed, Err(CgroupError::Malformed { .. })),
            "{text:?} parsed as {parsed:?}"
        );
    }
}

#[test]
fn unreclaimable_memory_is_anon_shmem_and_kernel() -> Result<(), CgroupError> {
    let text = "anon 1000\nfile 99999\nkernel 300\nshmem 20\nslab 7\n";
    assert_eq!(parse_unreclaimable(text)?, Bytes(1_320));
    Ok(())
}

#[test]
fn page_cache_is_not_counted() -> Result<(), CgroupError> {
    let text = "anon 5\nfile 1000000\nfile_mapped 50\nshmem 0\nkernel 0\n";
    assert_eq!(parse_unreclaimable(text)?, Bytes(5));
    Ok(())
}

#[test]
fn an_older_kernel_without_kernel_still_counts_anon_and_shmem()
-> Result<(), CgroupError> {
    assert_eq!(parse_unreclaimable("anon 10\nshmem 4\n")?, Bytes(14));
    Ok(())
}

#[test]
fn memory_stat_without_anon_is_missing_it() {
    let err = parse_unreclaimable("file 1\nshmem 2\n").err();
    assert_eq!(
        err,
        Some(CgroupError::Missing {
            file: "memory.stat",
            key: "anon"
        })
    );
}

#[test]
fn a_memory_stat_line_that_is_no_pair_is_malformed() {
    assert!(matches!(
        parse_unreclaimable("anon many\n"),
        Err(CgroupError::Malformed { .. })
    ));
}
