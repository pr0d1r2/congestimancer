//! `src/launch`: exec, no-cmd exit, govern, background priority, env cleanup.
//!
//! Scaffold: the node's code lands here (`src/launch` tasks). Each `use`
//! line below is one build-order edge for `sherd wave` (src:C); the
//! `expect` turns into a warning once the node uses them, and goes.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

use crate::gate as _;
use crate::signal as _;
use crate::slot as _;
use crate::sys as _;
use crate::wait as _;
