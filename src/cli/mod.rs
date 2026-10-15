//! `src/cli`: clap verbs, flags, env defaults, exit codes, completions, man page.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3).

#![expect(unused_imports, reason = "scaffold: build-order edges")]
#![allow(dead_code, reason = "parts land before the entry point wires them")]

mod args;
mod help;
mod switches;
mod table;
mod values;

pub use switches::SWITCHES;
pub use table::{SwitchSpec, env_name};

use crate::advise as _;
use crate::gate as _;
use crate::launch as _;
use crate::profile as _;
use crate::remote as _;
use crate::report as _;
use crate::signal as _;
use crate::slot as _;
use crate::units as _;
use crate::wait as _;
