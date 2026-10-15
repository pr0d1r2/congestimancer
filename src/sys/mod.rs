//! `src/sys`: OS facade: all syscalls, FFI, `sysinfo`, `rustix`, `libc`;
//! the only node allowed `unsafe` (src:C, nix:V11).
//!
//! Composes only (src:C). Every function here is safe (src/sys:V2): a
//! failure is an [`std::io::Error`], and a source this machine lacks is
//! [`std::io::ErrorKind::Unsupported`] with a reason (src/sys:V4). Raw
//! bytes from the OS are decoded by pure functions tested on captured
//! fixtures under `tests/fixtures/sys/` (src/sys:V3).

mod bus;
mod capture;
mod disk;
mod error;
mod files;
mod forward;
mod fs;
mod gpu;
mod group;
#[cfg(target_os = "macos")]
#[allow(unsafe_code, reason = "IOKit FFI (src/sys:C)")]
mod iokit;
mod iopolicy;
mod mach;
mod net;
mod notify;
mod nvml;
mod power;
mod probe;
mod props;
mod raw;
mod snapshot;
mod sysctl;
mod time;
mod users;

pub use bus::{SYSTEM_BUS_SOCKET, system_bus_connect};
pub use capture::capture;
pub use disk::{
    BlockCounters, BlockDevice, BlockStats, MountSource, block_devices,
    mount_source, physical_disk_stats,
};
pub use files::{list_dir, read_text};
pub use forward::{Blocked, ForwardSignal, Wake, block_forwarded};
pub use fs::{
    FsStats, device_of, inherit_in, nice, set_nice, statvfs,
    try_lock_exclusive, unlock,
};
pub use gpu::{GpuUsage, gpu_usage};
pub use group::{GroupSignal, signal_group};
pub use iopolicy::{IoPriority, io_priority, set_io_priority};
pub use mach::{CpuTicks, PsetLoad, VmStats, cpu_ticks, pset_load, vm_stats};
pub use net::{InterfaceCounters, LinkCounters, interface_counters};
pub use notify::{LOW_POWER_MODE, THERMAL_PRESSURE, notify_state};
pub use nvml::{NvmlGpu, nvml_gpus};
pub use power::{PowerState, hid_idle_ns, power_state};
pub use probe::tcp_connect_time;
pub use props::{PropValue, Props};
pub use snapshot::{LoadAverage, Memory, ProcessInfo, Snapshot};
pub use sysctl::{SwapUsage, swap_usage, sysctl_int};
pub use time::local_time;
pub use users::{current_uid, user_id, user_name};
