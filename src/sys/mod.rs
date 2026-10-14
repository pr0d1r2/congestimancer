//! `src/sys`: OS facade: all syscalls, FFI, `sysinfo`, `rustix`, `libc`; only node allowed `unsafe`.
//!
//! Scaffold: the node's code lands here (`src/sys` tasks). Each `use`
//! line below is one build-order edge for `sherd wave` (src:C); the
//! `expect` turns into a warning once the node uses them, and goes.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

use crate::signal as _;
use crate::units as _;
