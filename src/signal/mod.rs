//! `src/signal`: the common model of every congestion signal: what a
//! reading is, how it is sampled, how tests fake it.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3).

mod id;
mod reading;

pub use id::SignalId;
pub use reading::{Level, Measure, Reading, Reason};
