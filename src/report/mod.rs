//! `src/report`: `status` human & JSON, verbose reasons, heartbeat text, colour.
//!
//! Scaffold: the node's code lands here (`src/report` tasks). Each `use`
//! line below is one build-order edge for `sherd wave` (src:C); the
//! `expect` turns into a warning once the node uses them, and goes.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

use crate::gate as _;
use crate::signal as _;
use crate::units as _;
