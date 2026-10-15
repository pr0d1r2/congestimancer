//! `src/cpu`: load, run queue, CPU idle, CPU pressure, steal.
//!
//! Composes only (src:C): the logical CPU count and per-CPU arithmetic
//! in `count`, the Linux parsers in `parse`, the signals in `signal`.
//! Linux files are read through `crate::sys` under a
//! [`Root`](crate::signal::Root), so tests read `tests/fixtures/cpu/`;
//! macOS reads `sysinfo` and mach calls through `crate::sys`. A cgroup
//! scope lays its own `cpu.pressure` over these signals through
//! `crate::cgroup::Scoped`.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod count;
mod error;
mod parse;
mod signal;

pub use count::{cpu_count, load_per_cpu};
pub use error::CpuError;
pub use parse::{Loads, parse_loadavg};
pub use signal::CpuSignal;

use crate::cgroup as _;
