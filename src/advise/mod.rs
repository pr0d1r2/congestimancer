//! `src/advise`: `advise` verb: how many jobs fit now, what limits them, what to do.
//!
//! Scaffold: the node's code lands here (`src/advise` tasks). Each `use`
//! line below is one build-order edge for `sherd wave` (src:C); the
//! `expect` turns into a warning once the node uses them, and goes.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

use crate::cpu as _;
use crate::gate as _;
use crate::io as _;
use crate::mem as _;
use crate::report as _;
use crate::signal as _;
use crate::slot as _;
use crate::units as _;
