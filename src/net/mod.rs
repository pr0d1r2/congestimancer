//! `src/net`: interface rates, RTT probe.
//!
//! Composes only (src:C): the Linux parsers in `dev`, the per-interface
//! rate bases in `rates`, the rate signals in `signal`, the usage errors
//! in `error`.

mod dev;
mod error;
mod rates;
mod signal;

pub use dev::{NetDevError, linux_interfaces, parse_if_flags, parse_net_dev};
pub use error::NetError;
pub use signal::{NetSignal, selected};
