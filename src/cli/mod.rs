//! `src/cli`: clap verbs, flags, env defaults, exit codes, completions, man page.
//!
//! Scaffold: the node's code lands here (`src/cli` tasks). Each `use`
//! line below is one build-order edge for `sherd wave` (src:C); the
//! `expect` turns into a warning once the node uses them, and goes.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

use crate::advise as _;
use crate::gate as _;
use crate::launch as _;
use crate::profile as _;
use crate::remote as _;
use crate::report as _;
use crate::signal as _;
use crate::slot as _;
use crate::units as _;
use crate::wait as _;
