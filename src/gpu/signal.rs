//! [`GpuSignal`]: `--gpu` and `--gpu-mem` over every GPU or the one
//! `--gpu-device` picks (src/gpu:T4, V1 to V6).

use std::io;

use super::{Gpu, GpuError, amd_gpus, worst_busy, worst_free};
use crate::signal::{Measure, Reason, Root, Sample, Signal, SignalId};
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

/// The GPU signals: `--gpu` (utilisation, src/gpu:V1) and `--gpu-mem`
/// (free GPU memory, V2). Wrap it in
/// [`Sampled`](crate::signal::Sampled) for a
/// [`Source`](crate::signal::Source).
///
/// Linux reads NVIDIA GPUs through NVML, then AMD cards from sysfs;
/// macOS reads every `IOAccelerator` (R1). By default every GPU is read
/// and the worst wins: the busiest, the least free memory, among the GPUs
/// that report it (V3). No source is unsupported (V4); so is `--gpu-mem`
/// on Apple Silicon, which has no free GPU memory of its own (V6). Any
/// other signal is unsupported.
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

    /// Reads only the GPU numbered `index` (`--gpu-device`), counted from
    /// 0 in the order of [`GpuSignal::gpus`] (src/gpu:V3, V5).
    ///
    /// # Errors
    ///
    /// [`GpuError::UnknownDevice`] when the GPUs can be listed and `index`
    /// is past them: a usage error, exit 64. With no GPU source at all the
    /// index is kept, and the gate reads unsupported (V4).
    pub fn device(self, index: Count) -> Result<Self, GpuError> {
        if let Ok(all) = self.gpus() {
            let found = all.len();
            if pick(all, index).is_none() {
                return Err(GpuError::UnknownDevice { index, found });
            }
        }
        Ok(Self {
            device: Some(index),
            ..self
        })
    }

    /// Every GPU, numbered as `--gpu-device` counts them: on Linux NVIDIA
    /// in NVML order, then AMD by card number; on macOS in IOKit order.
    ///
    /// # Errors
    ///
    /// The reason for the user when no source has a GPU (src/gpu:V4).
    pub fn gpus(&self) -> Result<Vec<Gpu>, Reason> {
        match self.os {
            Os::Linux => self.linux_gpus(),
            Os::Macos => {
                let found = (self.iokit)().map_err(reason)?;
                Ok(found.iter().map(Gpu::from).collect())
            }
        }
    }

    /// NVIDIA through NVML, then AMD from sysfs; why neither has a GPU.
    fn linux_gpus(&self) -> Result<Vec<Gpu>, Reason> {
        let (mut gpus, nvml) = match (self.nvml)() {
            Ok(found) => (found.iter().map(Gpu::from).collect(), None),
            Err(err) => (Vec::new(), Some(err.to_string())),
        };
        gpus.extend(amd_gpus(&self.root));
        if gpus.is_empty() {
            let nvml = nvml.unwrap_or_else(|| "NVML: no NVIDIA GPU".into());
            return Err(Reason::from(format!(
                "no GPU: {nvml}; no AMD card under /sys/class/drm"
            )));
        }
        Ok(gpus)
    }

    /// The GPUs a gate reads now: all, or the `--gpu-device` one.
    fn chosen(&self) -> Result<Vec<Gpu>, Reason> {
        let all = self.gpus()?;
        let Some(index) = self.device else {
            return Ok(all);
        };
        let found = all.len();
        pick(all, index).map(|gpu| vec![gpu]).ok_or_else(|| {
            Reason::from(format!(
                "--gpu-device {index}: GPU is gone ({found} found)"
            ))
        })
    }

    /// `--gpu-mem`: the least free memory, or why no GPU reports it.
    fn free(&self) -> Result<Measure, Reason> {
        let gpus = self.chosen()?;
        worst_free(&gpus).map(Measure::Bytes).ok_or_else(|| {
            Reason::from(match self.os {
                Os::Linux => "no GPU reports its free memory",
                Os::Macos => {
                    "no GPU reports its free memory \
                     (Apple Silicon shares unified RAM)"
                }
            })
        })
    }
}

/// The GPU numbered `index` in `all`.
fn pick(all: Vec<Gpu>, index: Count) -> Option<Gpu> {
    let index = usize::try_from(index.0).ok()?;
    all.into_iter().nth(index)
}

impl Signal for GpuSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let sample = match id {
            SignalId::Gpu => self.chosen().and_then(|gpus| {
                worst_busy(&gpus).map(Measure::Percent).ok_or_else(|| {
                    Reason::from("no GPU reports its utilisation")
                })
            }),
            SignalId::GpuMem => self.free(),
            _ => Err(Reason::from(format!("{id} is not a GPU signal"))),
        };
        sample.map_or_else(Sample::Unsupported, Sample::Gauge)
    }
}

/// An error as the reason a signal has no value.
fn reason(err: impl ToString) -> Reason {
    Reason::from(err.to_string())
}

#[cfg(test)]
mod tests;
