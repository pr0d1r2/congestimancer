use crate::gpu::Gpu;
use crate::sys::{GpuUsage, Props};
use crate::units::{Bytes, Percent};

/// `n` whole percent.
fn pct(n: u16) -> Option<Percent> {
    n.checked_mul(100).and_then(Percent::from_hundredths)
}

#[test]
fn apple_silicon_reads_busy_without_free_memory() {
    let usage = GpuUsage {
        utilization_percent: Some(20),
        vram_free_bytes: None,
        stats: Props::new(),
    };
    assert_eq!(Gpu::from(&usage), Gpu::new(pct(20), None));
}

#[test]
fn a_discrete_mac_gpu_reads_free_vram() {
    let usage = GpuUsage {
        utilization_percent: Some(99),
        vram_free_bytes: Some(3 << 30),
        stats: Props::new(),
    };
    assert_eq!(Gpu::from(&usage), Gpu::new(pct(99), Some(Bytes(3 << 30))));
}

#[test]
fn a_utilization_past_100_is_not_a_percent() {
    let usage = GpuUsage {
        utilization_percent: Some(250),
        ..GpuUsage::default()
    };
    assert_eq!(Gpu::from(&usage), Gpu::new(None, None));
}
