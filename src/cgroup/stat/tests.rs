use std::path::Path;
use std::time::Duration;

use super::{CpuStat, Throttling, parse_cpu_stat};
use crate::cgroup::CgroupError;

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
