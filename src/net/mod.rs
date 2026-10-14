//! `src/net`: interface rates, RTT probe.
//!
//! Scaffold: the node's code lands here (`src/net` tasks). Each `use`
//! line below is one build-order edge for `sherd wave` (src:C); the
//! `expect` turns into a warning once the node uses them, and goes.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

use crate::signal as _;
use crate::sys as _;
use crate::units as _;
