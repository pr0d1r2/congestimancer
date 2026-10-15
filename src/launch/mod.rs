//! `src/launch`: exec, no-cmd exit, govern, background priority, env cleanup.
//!
//! Composes only (src:C): each part lives in its own submodule and the
//! public surface is re-exported here and from the crate root (src:V3).

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod after;
mod env;
mod exec;
mod priority;
mod start;

pub use after::{Priority, after_wait};
pub use env::clean_env;
pub use exec::{exec_command, exec_failure_code};
pub use priority::{BACKGROUND_NICE, background};
pub use start::{Start, launch};

use crate::gate as _;
use crate::signal as _;
use crate::slot as _;
use crate::sys as _;
use crate::wait as _;
