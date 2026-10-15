//! `src/procs`: process counts by name & by user.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3).

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod glob;

pub use glob::Glob;

use crate::signal as _;
use crate::sys as _;
use crate::units as _;
