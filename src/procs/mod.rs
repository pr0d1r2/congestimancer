//! `src/procs`: process counts by name & by user.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3).

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod error;
mod glob;
mod limit;

pub use error::ProcsError;
pub use glob::Glob;
pub use limit::{ProcLimit, parse_proc_limit};

use crate::signal as _;
use crate::sys as _;
