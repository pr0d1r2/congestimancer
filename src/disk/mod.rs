//! `src/disk`: free space & inodes on a path.
//!
//! Composes only (src:C): the path error in `error`, the gates in
//! `signal`, read through `crate::sys::statvfs` on both OSes
//! (src/disk:V4).

mod error;
mod signal;

pub use error::DiskError;
pub use signal::{DiskSignal, free_inodes, free_space};
