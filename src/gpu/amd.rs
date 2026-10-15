//! AMD GPUs from Linux sysfs: `gpu_busy_percent` and
//! `mem_info_vram_total|used` under `/sys/class/drm/card<N>/device`
//! (src/gpu:T1, V1, V2, V5).

use super::device::whole_percent;
use super::{Gpu, GpuError};
use crate::signal::Root;
use crate::sys::{list_dir, read_text};
use crate::units::{Bytes, Percent};

/// The DRM class: one `card<N>` entry per GPU.
const DRM_DIR: &str = "/sys/class/drm";

/// `gpu_busy_percent`: a whole percent, 0 to 100.
///
/// # Errors
///
/// [`GpuError::Malformed`] for anything but a whole number up to 100.
pub fn parse_busy_percent(text: &str) -> Result<Percent, GpuError> {
    digits(text)
        .and_then(whole_percent)
        .ok_or_else(|| GpuError::malformed("gpu_busy_percent", text))
}

/// `mem_info_vram_total` or `mem_info_vram_used` (`file`): bytes.
///
/// # Errors
///
/// [`GpuError::Malformed`] for anything but a whole number of bytes.
pub fn parse_vram(file: &'static str, text: &str) -> Result<Bytes, GpuError> {
    digits(text)
        .map(Bytes)
        .ok_or_else(|| GpuError::malformed(file, text))
}

/// The trimmed `text` as a whole number: digits alone, no sign.
fn digits(text: &str) -> Option<u64> {
    let text = text.trim();
    let all = !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit());
    all.then(|| text.parse().ok()).flatten()
}

/// Every AMD GPU under `root`'s `/sys/class/drm`, by card number: a
/// `card<N>` whose device has `gpu_busy_percent` or `mem_info_vram_total`.
/// Connectors (`card0-DP-1`), render nodes and cards with neither are
/// skipped; a file that does not parse, or used past total, leaves its
/// field `None` (src/gpu:V1, V2). Paths are built only from listed
/// entries, never from user text (src/gpu:V5).
#[must_use]
pub fn amd_gpus(root: &Root) -> Vec<Gpu> {
    let names = list_dir(root, DRM_DIR).unwrap_or_default();
    let mut cards: Vec<u64> = names.iter().filter_map(|n| card(n)).collect();
    cards.sort_unstable();
    cards.into_iter().filter_map(|n| amd_gpu(root, n)).collect()
}

/// The number of a `card<N>` entry; `None` for any other entry.
fn card(name: &str) -> Option<u64> {
    name.strip_prefix("card").and_then(digits)
}

/// Card `n`'s readings, if its device is an AMD GPU.
fn amd_gpu(root: &Root, n: u64) -> Option<Gpu> {
    let read = |file: &str| {
        read_text(root, &format!("{DRM_DIR}/card{n}/device/{file}")).ok()
    };
    let busy = read("gpu_busy_percent");
    let total = read("mem_info_vram_total");
    if busy.is_none() && total.is_none() {
        return None;
    }
    Some(Gpu::new(
        busy.and_then(|text| parse_busy_percent(&text).ok()),
        free_vram(total.as_deref(), read("mem_info_vram_used").as_deref()),
    ))
}

/// Total minus used VRAM; `None` unless both parse and used <= total.
fn free_vram(total: Option<&str>, used: Option<&str>) -> Option<Bytes> {
    let total = parse_vram("mem_info_vram_total", total?).ok()?;
    let used = parse_vram("mem_info_vram_used", used?).ok()?;
    total.0.checked_sub(used.0).map(Bytes)
}

#[cfg(test)]
mod tests;
