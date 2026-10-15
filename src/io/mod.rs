//! `src/io`: IO throughput, busy, latency, queue, pressure; path to device
//! resolution.
//!
//! Composes only (src:C): the errors in `error`, the Linux
//! `/proc/diskstats` parser in `diskstats`, path to
//! device resolution in `mount`.

mod diskstats;
mod error;
mod mount;

pub use diskstats::{Direction, DiskStat, parse_diskstats};
pub use error::IoError;
pub use mount::{Mount, linux_device, macos_device, mount_of, parse_mountinfo};
