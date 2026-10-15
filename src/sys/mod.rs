//! `src/sys`: OS facade: all syscalls, FFI, `sysinfo`, `rustix`, `libc`;
//! the only node allowed `unsafe` (src:C, nix:V11).
//!
//! Composes only (src:C). Every function here is safe (src/sys:V2): a
//! failure is an [`std::io::Error`], and a source this machine lacks is
//! [`std::io::ErrorKind::Unsupported`] with a reason (src/sys:V4). Raw
//! bytes from the OS are decoded by pure functions tested on captured
//! fixtures under `tests/fixtures/sys/` (src/sys:V3).

mod error;
mod files;
mod fs;
mod snapshot;

pub use files::read_text;
pub use fs::{FsStats, nice, set_nice, statvfs, try_lock_exclusive, unlock};
pub use snapshot::{LoadAverage, Memory, ProcessInfo, Snapshot};
