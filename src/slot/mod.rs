//! `src/slot`: per-user lock dir, slots, FIFO, rate, spacing, memory reservations, nested held marker.
//!
//! Composes only (src:C). State is files and `flock`s in one per-user
//! lock dir, never a daemon (src/slot:C): a holder that dies drops its
//! locks with its open files, so nothing is left to clean but names.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod dir;
mod error;
mod key;
#[cfg(test)]
mod scratch;

pub use dir::{DirFacts, LockDir, lock_dir_path, unsafe_reason};
pub use error::SlotError;
pub use key::slot_key;

use crate::signal as _;
use crate::sys as _;
use crate::units as _;
