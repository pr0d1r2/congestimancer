//! `src/slot`: per-user lock dir, slots, FIFO, rate, spacing, memory reservations, nested held marker.
//!
//! Composes only (src:C). State is files and `flock`s in one per-user
//! lock dir, never a daemon (src/slot:C): a holder that dies drops its
//! locks with its open files, so nothing is left to clean but names.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod dir;
mod error;
mod fifo;
mod file;
mod key;
#[cfg(test)]
mod scratch;
mod slots;

pub use dir::{DirFacts, LockDir, lock_dir_path, unsafe_reason};
pub use error::SlotError;
pub use fifo::Ticket;
pub use key::slot_key;
pub use slots::{SlotGuard, claim_slot, live_holders};

use crate::signal as _;
use crate::sys as _;
use crate::units as _;
