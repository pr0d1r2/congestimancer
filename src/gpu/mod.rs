//! `src/gpu`: GPU utilisation & free memory.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3). Linux AMD files are read through `crate::sys` under a
//! [`Root`](crate::signal::Root), so tests read `tests/fixtures/gpu/`;
//! NVIDIA (NVML) and macOS (IOKit) readings come from `crate::sys`.

mod amd;
mod device;
mod error;

pub use amd::{amd_gpus, parse_busy_percent, parse_vram};
pub use device::Gpu;
pub use error::GpuError;
