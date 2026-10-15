//! NVIDIA GPUs: an [`NvmlGpu`] from `crate::sys` as a [`Gpu`]
//! (src/gpu:T2, C, V1, V2). NVML is loaded at run time by `crate::sys`;
//! built without the `nvml` feature it answers unsupported.

use super::Gpu;
use super::device::whole_percent;
use crate::sys::NvmlGpu;
use crate::units::Bytes;

/// NVML's busy percent and free memory; a reading NVML did not give, or
/// a busy percent past 100, stays `None`.
impl From<&NvmlGpu> for Gpu {
    fn from(gpu: &NvmlGpu) -> Self {
        Self::new(
            gpu.utilization_percent
                .and_then(|n| whole_percent(u64::from(n))),
            gpu.memory_free.map(Bytes),
        )
    }
}

#[cfg(test)]
mod tests;
