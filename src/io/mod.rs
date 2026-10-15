//! `src/io`: IO throughput, busy, latency, queue, pressure; path to device
//! resolution.
//!
//! Composes only (src:C): the errors in `error`, the Linux
//! `/proc/diskstats` parser in `diskstats`.

mod diskstats;
mod error;

pub use diskstats::{Direction, DiskStat, parse_diskstats};
pub use error::IoError;
