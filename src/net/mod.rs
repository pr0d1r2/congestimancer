//! `src/net`: interface rates, RTT probe.
//!
//! Composes only (src:C): the Linux parsers in `dev`.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod dev;

use crate::signal as _;
use crate::sys as _;
use crate::units as _;

pub use dev::{NetDevError, linux_interfaces, parse_if_flags, parse_net_dev};
