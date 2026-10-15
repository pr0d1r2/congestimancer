//! `src/cpu`: load, run queue, CPU idle, CPU pressure, steal.
//!
//! Composes only (src:C): the logical CPU count and per-CPU arithmetic
//! in `count`.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod count;

pub use count::{cpu_count, load_per_cpu};

use crate::cgroup as _;
use crate::signal as _;
use crate::sys as _;
