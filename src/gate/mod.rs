//! `src/gate`: gate model, strict compare, AND evaluation, unsupported verdict, custom gates.
//!
//! Scaffold: the node's code lands here (`src/gate` tasks). Each `use`
//! line below is one build-order edge for `sherd wave` (src:C); the
//! `expect` turns into a warning once the node uses them, and goes.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

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
