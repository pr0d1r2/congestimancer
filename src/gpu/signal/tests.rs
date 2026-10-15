use std::io;
use std::path::Path;
use std::time::Instant;

use super::{GpuSignal, Os};
use crate::gpu::{Gpu, GpuError};
use crate::signal::{
    Measure, Reading, Root, Sample, Sampled, Signal, SignalId, Source,
};
use crate::sys::{GpuUsage, NvmlGpu, Props};
use crate::units::{Bytes, Count, Percent};

const GIB: u64 = 1 << 30;

/// The fixture tree `tests/fixtures/gpu/<case>`, laid out like `/`.
fn root(case: &str) -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/gpu").join(case))
}

/// `n` whole percent.
fn pct(n: u16) -> Option<Percent> {
    n.checked_mul(100).and_then(Percent::from_hundredths)
}

/// A busy gauge of `n` percent.
fn busy(n: u16) -> Option<Sample> {
    pct(n).map(|p| Sample::Gauge(Measure::Percent(p)))
}

/// A free memory gauge.
const fn free(bytes: u64) -> Sample {
    Sample::Gauge(Measure::Bytes(Bytes(bytes)))
}

/// The reason of an unsupported sample, if it is one.
fn reason(sample: Sample) -> Option<String> {
    match sample {
        Sample::Unsupported(reason) => Some(reason.as_str().to_owned()),
        Sample::Gauge(_) | Sample::Counter(_) => None,
    }
}

fn no_nvml() -> io::Result<Vec<NvmlGpu>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "NVML: no libnvidia-ml on this machine",
    ))
}

fn two_nvidia() -> io::Result<Vec<NvmlGpu>> {
    let gpu = |index, util, free| NvmlGpu {
        index,
        utilization_percent: Some(util),
        memory_total: Some(24 * GIB),
        memory_free: Some(free),
    };
    Ok(vec![gpu(0, 10, GIB), gpu(1, 95, 20 * GIB)])
}

fn m4() -> io::Result<Vec<GpuUsage>> {
    Ok(vec![GpuUsage {
        utilization_percent: Some(20),
        vram_free_bytes: None,
        stats: Props::new(),
    }])
}

fn intel_mac() -> io::Result<Vec<GpuUsage>> {
    let gpu = |util, vram| GpuUsage {
        utilization_percent: Some(util),
        vram_free_bytes: vram,
        stats: Props::new(),
    };
    Ok(vec![gpu(3, None), gpu(60, Some(2 * GIB))])
}

fn no_iokit() -> io::Result<Vec<GpuUsage>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "IOAccelerator: no PerformanceStatistics",
    ))
}

/// A Linux GPU signal over fixture `case`, NVIDIA from `nvml`.
fn linux(case: &str, nvml: super::NvmlRead) -> GpuSignal {
    GpuSignal {
        nvml,
        ..GpuSignal::on(root(case), Os::Linux)
    }
}

/// A macOS GPU signal, IOKit from `iokit`.
fn macos(iokit: super::IokitRead) -> GpuSignal {
    GpuSignal {
        iokit,
        ..GpuSignal::on(root("missing"), Os::Macos)
    }
}

#[test]
fn amd_cards_alone_worst_wins() {
    let mut gpu = linux("amd", no_nvml);
    assert_eq!(Some(gpu.sample(SignalId::Gpu)), busy(90));
    assert_eq!(gpu.sample(SignalId::GpuMem), free(GIB / 2));
}

#[test]
fn nvidia_come_first_then_amd_by_card() {
    let gpu = linux("amd", two_nvidia);
    let found = gpu.gpus().unwrap_or_default();
    let want = [
        Gpu::new(pct(10), Some(Bytes(GIB))),
        Gpu::new(pct(95), Some(Bytes(20 * GIB))),
        Gpu::new(pct(42), Some(Bytes(6 * GIB))),
        Gpu::new(pct(90), None),
        Gpu::new(pct(7), Some(Bytes(GIB / 2))),
    ];
    assert_eq!(found, want);
}

#[test]
fn every_gpu_is_read_by_default() {
    let mut gpu = linux("amd", two_nvidia);
    assert_eq!(Some(gpu.sample(SignalId::Gpu)), busy(95));
    assert_eq!(gpu.sample(SignalId::GpuMem), free(GIB / 2));
}

#[test]
fn gpu_device_reads_one_gpu_by_index() {
    let gpu = linux("amd", two_nvidia).device(Count(2));
    let mut gpu = gpu.unwrap_or_else(|_| linux("none", no_nvml));
    assert_eq!(Some(gpu.sample(SignalId::Gpu)), busy(42));
    assert_eq!(gpu.sample(SignalId::GpuMem), free(6 * GIB));
}

#[test]
fn gpu_device_past_the_gpus_is_a_usage_error() {
    let err = linux("amd", no_nvml).device(Count(3)).err();
    let want = GpuError::UnknownDevice {
        index: Count(3),
        found: 3,
    };
    assert_eq!(err, Some(want));
    let huge = linux("amd", no_nvml).device(Count(u64::MAX)).err();
    assert!(matches!(huge, Some(GpuError::UnknownDevice { .. })));
}

#[test]
fn gpu_device_without_any_source_waits_for_the_sample() {
    let gpu = linux("none", no_nvml).device(Count(0));
    let mut gpu = gpu.unwrap_or_else(|_| linux("amd", two_nvidia));
    assert!(reason(gpu.sample(SignalId::Gpu)).is_some());
}

#[test]
fn a_gpu_that_went_away_is_unsupported() {
    let mut gpu = GpuSignal {
        device: Some(Count(4)),
        ..linux("amd", no_nvml)
    };
    let why = reason(gpu.sample(SignalId::Gpu)).unwrap_or_default();
    assert_eq!(why, "--gpu-device 4: GPU is gone (3 found)");
}

#[test]
fn no_gpu_source_is_unsupported_with_both_reasons() {
    let mut gpu = linux("none", no_nvml);
    let why = reason(gpu.sample(SignalId::GpuMem)).unwrap_or_default();
    assert_eq!(
        why,
        "no GPU: NVML: no libnvidia-ml on this machine; \
         no AMD card under /sys/class/drm"
    );
}

#[test]
fn a_gpu_that_reports_nothing_is_unsupported() {
    let mut gpu = linux("amd-odd", no_nvml);
    let busy = reason(gpu.sample(SignalId::Gpu)).unwrap_or_default();
    assert_eq!(busy, "no GPU reports its utilisation");
    let mem = reason(gpu.sample(SignalId::GpuMem)).unwrap_or_default();
    assert_eq!(mem, "no GPU reports its free memory");
}

#[test]
fn apple_silicon_reads_busy_but_not_free_memory() {
    let mut gpu = macos(m4);
    assert_eq!(Some(gpu.sample(SignalId::Gpu)), busy(20));
    let why = reason(gpu.sample(SignalId::GpuMem)).unwrap_or_default();
    assert_eq!(
        why,
        "no GPU reports its free memory (Apple Silicon shares unified RAM)"
    );
}

#[test]
fn an_intel_mac_reads_its_discrete_gpu() {
    let mut gpu = macos(intel_mac);
    assert_eq!(Some(gpu.sample(SignalId::Gpu)), busy(60));
    assert_eq!(gpu.sample(SignalId::GpuMem), free(2 * GIB));
    let one = macos(intel_mac).device(Count(0));
    let mut one = one.unwrap_or_else(|_| macos(m4));
    assert_eq!(Some(one.sample(SignalId::Gpu)), busy(3));
}

#[test]
fn a_mac_without_statistics_is_unsupported() {
    let mut gpu = macos(no_iokit);
    let why = reason(gpu.sample(SignalId::Gpu)).unwrap_or_default();
    assert_eq!(why, "IOAccelerator: no PerformanceStatistics");
}

#[test]
fn other_signals_are_not_gpu_signals() {
    let why = reason(macos(m4).sample(SignalId::Load)).unwrap_or_default();
    assert_eq!(why, "load is not a GPU signal");
}

#[test]
fn gpu_readings_are_gauges_with_no_warm_up() {
    let mut source = Sampled::new(macos(m4));
    let reading = source.read(SignalId::Gpu, Instant::now());
    assert_eq!(reading.measure(), busy(20).as_ref().and_then(gauge));
}

/// The measure of a gauge sample.
const fn gauge(sample: &Sample) -> Option<&Measure> {
    match sample {
        Sample::Gauge(measure) => Some(measure),
        Sample::Counter(_) | Sample::Unsupported(_) => None,
    }
}

#[test]
fn this_machine_reads_a_gauge_or_unsupported() {
    let mut gpu = GpuSignal::new(Root::system());
    for id in [SignalId::Gpu, SignalId::GpuMem] {
        let sample = gpu.sample(id);
        assert!(
            matches!(sample, Sample::Gauge(_) | Sample::Unsupported(_)),
            "{id}: {sample:?}"
        );
    }
    assert!(!matches!(
        Sampled::new(gpu).read(SignalId::Gpu, Instant::now()),
        Reading::Warming
    ));
}
