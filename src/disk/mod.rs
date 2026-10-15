//! `src/disk`: free space & inodes on a path.
//!
//! Composes only (src:C): the gates in `signal`, read through
//! `crate::sys::statvfs` on both OSes (src/disk:V4).

mod signal;

pub use signal::{DiskSignal, free_inodes, free_space};
