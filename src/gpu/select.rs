//! Worst wins: the GPU readings a gate compares when it reads every GPU
//! (src/gpu:T4, V3).

use super::Gpu;
use crate::units::{Bytes, Percent};

/// `--gpu`: the busiest GPU's utilisation; GPUs that do not report one are
/// passed over, and `None` when none does.
#[must_use]
pub fn worst_busy(gpus: &[Gpu]) -> Option<Percent> {
    let _ = gpus;
    None
}

/// `--gpu-mem`: the least free GPU memory; GPUs that do not report it
/// (Apple Silicon, src/gpu:V6) are passed over, and `None` when none does.
#[must_use]
pub fn worst_free(gpus: &[Gpu]) -> Option<Bytes> {
    let _ = gpus;
    None
}

#[cfg(test)]
mod tests;
