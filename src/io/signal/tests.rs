use std::path::Path;
use std::time::Instant;

use super::{IoSignal, Os};
use crate::signal::{
    Counter, Reading, Root, Sample, Sampled, Signal, SignalId, Source,
};

/// The fixture tree `tests/fixtures/io/<case>`, laid out like `/`.
fn root(case: &str) -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/io").join(case))
}

/// An IO signal over fixture `case` reading `os`'s sources for the
/// device holding `path`.
fn on(case: &str, os: Os, path: &str) -> IoSignal {
    IoSignal::on(root(case), os, Some(Path::new(path)))
}

/// The Linux sample of `id` for `/work` in fixture `linux`.
fn linux(id: SignalId) -> Sample {
    on("linux", Os::Linux, "/work").sample(id)
}

/// The reason of an unsupported sample, if it is one.
fn reason(sample: Sample) -> Option<String> {
    match sample {
        Sample::Unsupported(reason) => Some(reason.as_str().to_owned()),
        Sample::Gauge(_) | Sample::Counter(_) => None,
    }
}

/// A rate counter of `total` bytes.
const fn rate(total: u64) -> Sample {
    Sample::Counter(Counter::Rate { total })
}

#[test]
fn the_host_os_is_picked_by_new() {
    let want = if cfg!(target_os = "macos") {
        Os::Macos
    } else {
        Os::Linux
    };
    assert_eq!(IoSignal::new(root("linux"), None).os, want);
}

#[test]
fn without_a_disk_path_the_working_directory_is_measured() {
    let io = IoSignal::new(root("linux"), None);
    assert_eq!(io.path, Path::new("."));
}

#[test]
fn a_signal_of_another_domain_is_unsupported() {
    assert_eq!(
        reason(linux(SignalId::Mem)),
        Some("mem is not an io signal".to_owned())
    );
}

#[test]
fn linux_read_and_write_count_the_paths_device_bytes() {
    assert_eq!(linux(SignalId::IoRead), rate(1_536_000));
    assert_eq!(linux(SignalId::IoWrite), rate(512_000));
    assert_eq!(linux(SignalId::Io), rate(2_048_000));
}

#[test]
fn the_first_rate_reading_is_warming() {
    let mut source = Sampled::new(on("linux", Os::Linux, "/work"));
    let reading = source.read(SignalId::IoRead, Instant::now());
    assert_eq!(reading, Reading::Warming);
}

#[test]
fn a_path_on_tmpfs_is_unsupported_with_a_hint() {
    let sample = on("tmp", Os::Linux, "/x").sample(SignalId::IoWrite);
    assert_eq!(
        reason(sample),
        Some(
            "`/x` is on tmpfs, not a block device; name one with --io-device"
                .to_owned()
        )
    );
}

#[test]
fn linux_without_diskstats_is_unsupported() {
    let sample = on("absent", Os::Linux, "/").sample(SignalId::Io);
    assert_eq!(
        reason(sample),
        Some("/proc/diskstats: not present".to_owned())
    );
}

#[test]
fn garbled_linux_files_are_unsupported() {
    let sample = on("garbled", Os::Linux, "/").sample(SignalId::Io);
    let text = reason(sample).unwrap_or_default();
    assert!(text.starts_with("malformed /proc/diskstats line"), "{text}");
}

#[test]
fn macos_reads_the_disk_holding_the_path() {
    let sample = on("absent", Os::Macos, "/").sample(SignalId::IoRead);
    let counted = matches!(sample, Sample::Counter(Counter::Rate { .. }));
    assert_eq!(counted, cfg!(target_os = "macos"), "{sample:?}");
}

/// `--io-busy` and `--io-latency` (src/io:T5, T6).
mod busy_and_latency {
    use std::time::Duration;

    use super::super::{Totals, block_totals, counter};
    use super::{Counter, Sample, SignalId, linux};
    use crate::sys::{BlockCounters, BlockStats};

    #[test]
    fn linux_busy_counts_io_ticks() {
        let busy = Duration::from_millis(600);
        let want = Sample::Counter(Counter::Busy { busy });
        assert_eq!(linux(SignalId::IoBusy), want);
    }

    #[test]
    fn linux_latency_counts_time_over_completed_requests() {
        let time = Duration::from_millis(650);
        let mean = Counter::Mean { time, events: 160 };
        assert_eq!(linux(SignalId::IoLatency), Sample::Counter(mean));
    }

    /// Counters of `operations` taking `total_time_ns`.
    fn counters(operations: u64, total_time_ns: u64) -> BlockCounters {
        BlockCounters {
            bytes: 10,
            operations,
            total_time_ns,
            ..BlockCounters::default()
        }
    }

    #[test]
    fn macos_busy_and_latency_count_total_time() {
        let (read, write) = (counters(3, 5_000), counters(1, 7_000));
        let totals = block_totals(&BlockStats { read, write });
        let time = Duration::from_nanos(12_000);
        let got = totals.ok().map(|t| (t.busy, t.time, t.ops));
        assert_eq!(got, Some((time, time, 4)));
    }

    #[test]
    fn macos_counters_past_64_bits_are_unsupported() {
        let (read, write) = (counters(1, u64::MAX), counters(1, 1));
        assert!(block_totals(&BlockStats { read, write }).is_err());
    }

    /// Totals of `read` and `written` bytes, nothing else.
    const fn bytes(read: u64, written: u64) -> Totals {
        Totals {
            read,
            written,
            busy: Duration::ZERO,
            time: Duration::ZERO,
            ops: 0,
        }
    }

    #[test]
    fn an_io_sum_past_64_bits_is_unsupported() {
        let sum = counter(SignalId::Io, &bytes(u64::MAX, 1));
        let sum = sum.map_err(|r| r.to_string());
        assert_eq!(sum, Err("IO bytes past 64 bits".to_owned()));
    }

    #[test]
    fn a_gauge_is_no_io_counter() {
        let other = counter(SignalId::IoQueue, &bytes(0, 0));
        let other = other.map_err(|r| r.to_string());
        assert_eq!(other, Err("io-queue is not an io counter".to_owned()));
    }
}

/// `--io-queue` (src/io:T7).
mod queue {
    use super::{Os, Sample, Signal, SignalId, linux, on, reason};
    use crate::signal::Measure;
    use crate::units::Count;

    #[test]
    fn linux_queue_is_the_requests_in_flight() {
        let queue = Sample::Gauge(Measure::Count(Count(1)));
        assert_eq!(linux(SignalId::IoQueue), queue);
    }

    #[test]
    fn linux_queue_on_tmpfs_is_unsupported() {
        let sample = on("tmp", Os::Linux, "/x").sample(SignalId::IoQueue);
        assert!(reason(sample).is_some());
    }

    #[test]
    fn macos_queue_is_unsupported() {
        let sample = on("linux", Os::Macos, "/").sample(SignalId::IoQueue);
        let want = Some("no in-flight count on macOS".to_owned());
        assert_eq!(reason(sample), want);
    }
}
