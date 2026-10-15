//! `src/mem`: available memory, memory pressure & level, swap rate, dirty pages.
//!
//! Composes only (src:C): the gates' arithmetic in `fit`, the signals in
//! `signal`. Linux files are read through `crate::sys` under a
//! [`Root`](crate::signal::Root), so tests read `tests/fixtures/mem/`;
//! macOS reads sysctls and mach counters through `crate::sys`. A cgroup
//! scope lays its own memory and pressure over these signals through
//! `crate::cgroup::Scoped`.

mod error;
mod fit;
mod level;
mod parse;
mod signal;

pub use error::MemError;
pub use fit::{mem_per_cpu, mem_unreserved};
pub use level::{available_level, level_within, pressure_level};
pub use parse::{
    DirtyPages, SwapPages, parse_meminfo, parse_page_size, parse_vmstat,
};
pub use signal::MemSignal;
