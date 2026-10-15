//! `src/sys`: OS facade: all syscalls, FFI, `sysinfo`, `rustix`, `libc`;
//! the only node allowed `unsafe` (src:C, nix:V11).
//!
//! Composes only (src:C). Every function here is safe (src/sys:V2): a
//! failure is an [`std::io::Error`], and a source this machine lacks is
//! [`std::io::ErrorKind::Unsupported`] with a reason (src/sys:V4). Raw
//! bytes from the OS are decoded by pure functions tested on captured
//! fixtures under `tests/fixtures/sys/` (src/sys:V3).

mod disk;
mod error;
mod files;
mod fs;
#[cfg(target_os = "macos")]
#[allow(unsafe_code, reason = "IOKit FFI (src/sys:C)")]
mod iokit;
mod mach;
mod notify;
mod power;
mod props;
mod raw;
mod snapshot;
mod sysctl;

pub use disk::{
    BlockCounters, BlockDevice, BlockStats, MountSource, block_devices,
    mount_source, physical_disk_stats,
};
pub use files::read_text;
pub use fs::{FsStats, nice, set_nice, statvfs, try_lock_exclusive, unlock};
pub use mach::{CpuTicks, PsetLoad, VmStats, cpu_ticks, pset_load, vm_stats};
pub use notify::{LOW_POWER_MODE, THERMAL_PRESSURE, notify_state};
pub use power::{PowerState, hid_idle_ns, power_state};
pub use props::{PropValue, Props};
pub use snapshot::{LoadAverage, Memory, ProcessInfo, Snapshot};
pub use sysctl::{SwapUsage, swap_usage, sysctl_int};
