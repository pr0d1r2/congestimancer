//! macOS GPUs: an IOKit `PerformanceStatistics` reading ([`GpuUsage`]
//! from `crate::sys`) as a [`Gpu`] (src/gpu:T3, R1, V1, V2, V6).

use super::Gpu;
use super::device::whole_percent;
use crate::sys::GpuUsage;
use crate::units::Bytes;

/// `Device Utilization %` and `vramFreeBytes`. Apple Silicon has no
/// `vramFreeBytes` (unified memory), so its free memory stays `None` and
/// `--gpu-mem` is unsupported while `--gpu` still reads (src/gpu:V6).
impl From<&GpuUsage> for Gpu {
    fn from(gpu: &GpuUsage) -> Self {
        Self::new(
            gpu.utilization_percent.and_then(whole_percent),
            gpu.vram_free_bytes.map(Bytes),
        )
    }
}

#[cfg(test)]
mod tests;
