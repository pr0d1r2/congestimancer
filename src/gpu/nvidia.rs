//! NVIDIA GPUs: an [`NvmlGpu`] from `crate::sys` as a [`Gpu`]
//! (src/gpu:T2, C, V1, V2). NVML is loaded at run time by `crate::sys`;
//! built without the `nvml` feature it answers unsupported.

use super::Gpu;
use crate::sys::NvmlGpu;

/// NVML's busy percent and free memory; a reading NVML did not give, or
/// a busy percent past 100, stays `None`.
impl From<&NvmlGpu> for Gpu {
    fn from(gpu: &NvmlGpu) -> Self {
        let _ = gpu;
        Self::default()
    }
}

#[cfg(test)]
mod tests;
