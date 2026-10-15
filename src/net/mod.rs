//! `src/net`: interface rates, RTT probe.
//!
//! Composes only (src:C): the Linux parsers in `dev`, the per-interface
//! rate bases in `rates`, the rate signals in `signal`.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod dev;
mod rates;
mod signal;

use crate::units as _;

pub use dev::{NetDevError, linux_interfaces, parse_if_flags, parse_net_dev};
pub use signal::{NetSignal, up_interfaces};
