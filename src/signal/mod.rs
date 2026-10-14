//! `src/signal`: `Signal` & `Source` traits, `Reading`, sampler w/ two-sample deltas, monotonic clock, fixture root.
//!
//! Scaffold: the node's code lands here (`src/signal` tasks). Each `use`
//! line below is one build-order edge for `sherd wave` (src:C); the
//! `expect` turns into a warning once the node uses them, and goes.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

use crate::units as _;
