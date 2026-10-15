//! `src/cgroup`: cgroup v2 detection, limits, throttling, pressure override.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3). Every file is read through `crate::sys` under a
//! [`Root`](crate::signal::Root), so tests read `tests/fixtures/cgroup/`.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod error;
mod membership;

pub use error::CgroupError;
pub use membership::{Cgroup, parse_proc_cgroup};

use crate::signal as _;
use crate::sys as _;
use crate::units as _;
