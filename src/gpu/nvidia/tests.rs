use crate::gpu::Gpu;
use crate::sys::NvmlGpu;
use crate::units::{Bytes, Percent};

/// `n` whole percent.
fn pct(n: u16) -> Option<Percent> {
    n.checked_mul(100).and_then(Percent::from_hundredths)
}

#[test]
fn nvml_busy_and_free_memory_are_read() {
    let nvml = NvmlGpu {
        index: 1,
        utilization_percent: Some(63),
        memory_total: Some(24 << 30),
        memory_free: Some(10 << 30),
    };
    assert_eq!(Gpu::from(&nvml), Gpu::new(pct(63), Some(Bytes(10 << 30))));
}

#[test]
fn readings_nvml_did_not_give_stay_empty() {
    let nvml = NvmlGpu {
        index: 0,
        utilization_percent: None,
        memory_total: None,
        memory_free: None,
    };
    assert_eq!(Gpu::from(&nvml), Gpu::new(None, None));
}

#[test]
fn a_busy_percent_past_100_is_not_a_percent() {
    let nvml = NvmlGpu {
        utilization_percent: Some(101),
        memory_free: Some(0),
        ..NvmlGpu::default()
    };
    assert_eq!(Gpu::from(&nvml), Gpu::new(None, Some(Bytes(0))));
}
