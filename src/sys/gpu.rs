//! macOS GPU load: every `IOAccelerator` service's
//! `PerformanceStatistics` (src/sys:T9, V1, V3, V4; src/gpu:R1). Every
//! other OS answers [`io::ErrorKind::Unsupported`]; Linux reads NVML and
//! sysfs.

use std::io;

use super::props::Props;

/// One GPU's load.
#[derive(Debug, Clone, PartialEq, Default)]
#[non_exhaustive]
pub struct GpuUsage {
    /// `Device Utilization %`, 0 to 100.
    pub utilization_percent: Option<u64>,
    /// `vramFreeBytes`: free dedicated memory; `None` on unified memory.
    pub vram_free_bytes: Option<u64>,
    /// The whole `PerformanceStatistics` table, for keys not lifted above.
    pub stats: Props,
}

/// Every GPU's load, in IOKit order.
///
/// # Errors
///
/// Unsupported when no `IOAccelerator` has `PerformanceStatistics`, or off
/// macOS.
pub fn gpu_usage() -> io::Result<Vec<GpuUsage>> {
    let tables = imp::performance_statistics()?;
    if tables.is_empty() {
        return Err(super::error::unsupported(
            "IOAccelerator: no PerformanceStatistics",
        ));
    }
    Ok(tables.into_iter().map(decode_usage).collect())
}

/// One `PerformanceStatistics` table.
fn decode_usage(stats: Props) -> GpuUsage {
    GpuUsage {
        stats,
        ..GpuUsage::default()
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use std::io;

    use crate::sys::iokit::services;
    use crate::sys::props::Props;

    pub(super) fn performance_statistics() -> io::Result<Vec<Props>> {
        let accelerators = services(c"IOAccelerator")?;
        Ok(accelerators
            .iter()
            .filter_map(|gpu| gpu.props("PerformanceStatistics"))
            .collect())
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use std::io;

    use crate::sys::props::Props;

    pub(super) fn performance_statistics() -> io::Result<Vec<Props>> {
        Err(crate::sys::error::unsupported("IOAccelerator: macOS only"))
    }
}

#[cfg(test)]
mod tests;
