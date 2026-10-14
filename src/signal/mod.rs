//! `src/signal`: the common model of every congestion signal: what a
//! reading is, how it is sampled, how tests fake it.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3).
//!
//! Scaffold: each `use` line below is one build-order edge for
//! `sherd wave` (src:C); the `expect` goes once the node uses them.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod id;

pub use id::SignalId;

use crate::units as _;
