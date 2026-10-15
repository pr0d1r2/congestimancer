//! AMD GPUs from Linux sysfs: `gpu_busy_percent` and
//! `mem_info_vram_total|used` under `/sys/class/drm/card<N>/device`
//! (src/gpu:T1, V1, V2, V5).

use super::{Gpu, GpuError};
use crate::signal::Root;
use crate::units::{Bytes, Percent};

/// The DRM class: one `card<N>` entry per GPU.
pub(crate) const DRM_DIR: &str = "/sys/class/drm";

/// `gpu_busy_percent`: a whole percent, 0 to 100.
///
/// # Errors
///
/// [`GpuError::Malformed`] for anything but a whole number up to 100.
pub fn parse_busy_percent(text: &str) -> Result<Percent, GpuError> {
    Err(GpuError::malformed("gpu_busy_percent", text))
}

/// `mem_info_vram_total` or `mem_info_vram_used` (`file`): bytes.
///
/// # Errors
///
/// [`GpuError::Malformed`] for anything but a whole number of bytes.
pub fn parse_vram(file: &'static str, text: &str) -> Result<Bytes, GpuError> {
    Err(GpuError::malformed(file, text))
}

/// Every AMD GPU under `root`'s `/sys/class/drm`, by card number: a
/// `card<N>` whose device reports busy percent or VRAM. Connectors
/// (`card0-DP-1`), render nodes and cards with neither are skipped; a
/// file that does not parse leaves its field `None`.
#[must_use]
pub fn amd_gpus(root: &Root) -> Vec<Gpu> {
    let _ = (root, DRM_DIR, super::device::whole_percent);
    Vec::new()
}

#[cfg(test)]
mod tests;
