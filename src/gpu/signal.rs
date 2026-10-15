//! [`GpuSignal`]: `--gpu` and `--gpu-mem` over every GPU or the one
//! `--gpu-device` picks (src/gpu:T4, V1 to V6).

use std::io;

use super::{Gpu, GpuError};
use crate::signal::{Reason, Root, Sample, Signal, SignalId};
use crate::sys::{GpuUsage, NvmlGpu, gpu_usage, nvml_gpus};
use crate::units::Count;

/// Which OS's sources a [`GpuSignal`] reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
    /// NVML, then AMD cards in sysfs under the root.
    Linux,
    /// IOKit `PerformanceStatistics`.
    Macos,
}

/// Every NVIDIA GPU NVML sees (`crate::sys`, or a test double).
type NvmlRead = fn() -> io::Result<Vec<NvmlGpu>>;

/// Every IOKit GPU's statistics (`crate::sys`, or a test double).
type IokitRead = fn() -> io::Result<Vec<GpuUsage>>;

/// The GPU signals: `--gpu` (utilisation) and `--gpu-mem` (free GPU
/// memory). Wrap it in [`Sampled`](crate::signal::Sampled) for a
/// [`Source`](crate::signal::Source).
#[derive(Debug, Clone)]
pub struct GpuSignal {
    /// The directory that stands for `/`.
    root: Root,
    /// The OS whose sources are read.
    os: Os,
    /// The one GPU read (`--gpu-device`), or `None` for every GPU.
    device: Option<Count>,
    /// Where NVIDIA readings come from.
    nvml: NvmlRead,
    /// Where macOS readings come from.
    iokit: IokitRead,
}

impl GpuSignal {
    /// Reads this OS's sources, Linux files under `root`.
    #[must_use]
    pub fn new(root: Root) -> Self {
        let os = if cfg!(target_os = "macos") {
            Os::Macos
        } else {
            Os::Linux
        };
        Self::on(root, os)
    }

    /// Reads `os`'s sources through `crate::sys`.
    fn on(root: Root, os: Os) -> Self {
        Self {
            root,
            os,
            device: None,
            nvml: nvml_gpus,
            iokit: gpu_usage,
        }
    }

    /// Reads only the GPU numbered `index` (`--gpu-device`).
    ///
    /// # Errors
    ///
    /// [`GpuError::UnknownDevice`] when the GPUs can be listed and `index`
    /// is past them: a usage error, exit 64.
    pub fn device(self, index: Count) -> Result<Self, GpuError> {
        Ok(Self {
            device: Some(index),
            ..self
        })
    }

    /// Every GPU, numbered as `--gpu-device` counts them, or why there is
    /// none.
    ///
    /// # Errors
    ///
    /// The reason for the user when no source has a GPU (src/gpu:V4).
    pub fn gpus(&self) -> Result<Vec<Gpu>, Reason> {
        let _ = (&self.root, self.os, self.device, self.nvml, self.iokit);
        Err(Reason::from("not yet"))
    }
}

impl Signal for GpuSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        Sample::Unsupported(Reason::from(format!("{id}: not yet")))
    }
}

#[cfg(test)]
mod tests;
