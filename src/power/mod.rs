//! `src/power`: thermal throttling, temperature, AC, battery, low power.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3). Linux files are read through `crate::sys` under a
//! [`Root`](crate::signal::Root), so tests read `tests/fixtures/power/`;
//! macOS goes through `crate::sys` notify and IOKit calls.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod error;
mod thermal;

pub use error::PowerError;
pub use thermal::{
    is_cpu_cooling, parse_cpu_list, parse_millicelsius, parse_sysfs_u64,
};

use crate::signal as _;
use crate::sys as _;
