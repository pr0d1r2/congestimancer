//! `src/cgroup`: cgroup v2 detection, limits, throttling, pressure override.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3). Every file is read through `crate::sys` under a
//! [`Root`](crate::signal::Root), so tests read `tests/fixtures/cgroup/`.

mod error;
mod limits;
mod membership;
mod scope;
mod signal;
mod stat;

pub use error::CgroupError;
pub use limits::{
    CpuMax, memory_available, parse_cpu_max, parse_memory_current,
    parse_memory_max,
};
pub use membership::{Cgroup, parse_proc_cgroup};
pub use scope::{CgroupMode, Scope, parse_cgroup_mode};
pub use signal::CgroupSignal;
pub use stat::{CpuStat, Throttling, parse_cpu_stat};
