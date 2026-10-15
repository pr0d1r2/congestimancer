//! `src/cpu`: load, run queue, CPU idle, CPU pressure, steal.
//!
//! Composes only (src:C): the logical CPU count and per-CPU arithmetic
//! in `count`, `--load-window` in `window`, the Linux parsers in
//! `parse`, the signals in `signal`.
//! Linux files are read through `crate::sys` under a
//! [`Root`](crate::signal::Root), so tests read `tests/fixtures/cpu/`;
//! macOS reads `sysinfo` and mach calls through `crate::sys`. A cgroup
//! scope lays its own `cpu.pressure` over these signals through
//! `crate::cgroup::Scoped`.

mod count;
mod error;
mod parse;
mod signal;
mod window;

pub use count::{cpu_count, load_per_cpu};
pub use error::CpuError;
pub use parse::{CpuTimes, Loads, ProcStat, parse_loadavg, parse_stat};
pub use signal::CpuSignal;
pub use window::{LoadWindow, parse_load_window};
