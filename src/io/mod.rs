//! `src/io`: IO throughput, busy, latency, queue, pressure; path to device
//! resolution.
//!
//! Composes only (src:C): the errors in `error`, the Linux
//! `/proc/diskstats` parser in `diskstats`, path to
//! device resolution in `mount`, the signals in `signal`.
//! Linux files are read through `crate::sys` under a
//! [`Root`](crate::signal::Root), so tests read `tests/fixtures/io/`;
//! macOS reads IOKit through `crate::sys`. A cgroup scope lays its own
//! `io.pressure` over these signals through `crate::cgroup::Scoped`.

mod diskstats;
mod error;
mod mount;
mod signal;

pub use diskstats::{Direction, DiskStat, parse_diskstats};
pub use error::IoError;
pub use mount::{Mount, linux_device, macos_device, mount_of, parse_mountinfo};
pub use signal::{IoSignal, parse_io_device};
