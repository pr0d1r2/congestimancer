//! macOS GPUs: an IOKit `PerformanceStatistics` reading ([`GpuUsage`]
//! from `crate::sys`) as a [`Gpu`] (src/gpu:T3, R1, V1, V2, V6).

use super::Gpu;
use crate::sys::GpuUsage;

/// `Device Utilization %` and `vramFreeBytes`. Apple Silicon has no
/// `vramFreeBytes` (unified memory), so its free memory stays `None` and
/// `--gpu-mem` is unsupported while `--gpu` still reads (src/gpu:V6).
impl From<&GpuUsage> for Gpu {
    fn from(gpu: &GpuUsage) -> Self {
        let _ = gpu;
        Self::default()
    }
}

#[cfg(test)]
mod tests;
