//! `src/gate`: gate model, strict compare, AND evaluation, unsupported verdict, custom gates.
//!
//! Composes only (src:C): each part lives in its own submodule and the
//! public surface is re-exported here and from the crate root (src:V3).
//! The remaining `use crate::X as _;` lines are build-order edges for
//! `sherd wave` (src:C) to domain nodes the gate reads through a
//! [`Source`](crate::signal::Source), not by name.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod custom;
mod eval;
mod model;
mod once;

pub use custom::Custom;
pub use eval::{Checked, Evaluation, Gates, Overall, Subject, evaluate};
pub use model::{Adjust, Gate, Op, Threshold, Verdict};
pub use once::{evaluate_once, sample_once};

use crate::cgroup as _;
use crate::cpu as _;
use crate::disk as _;
use crate::gpu as _;
use crate::io as _;
use crate::limits as _;
use crate::mem as _;
use crate::net as _;
use crate::power as _;
use crate::presence as _;
use crate::procs as _;
use crate::signal as _;
use crate::units as _;
