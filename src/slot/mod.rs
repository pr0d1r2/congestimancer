//! `src/slot`: per-user lock dir, slots, FIFO, rate, spacing, memory reservations, nested held marker.
//!
//! Composes only (src:C). State is files and `flock`s in one per-user
//! lock dir, never a daemon (src/slot:C): a holder that dies drops its
//! locks with its open files, so nothing is left to clean but names.

mod crew;
mod dir;
mod error;
mod fifo;
mod file;
mod held;
mod key;
mod order;
mod reserve;
#[cfg(test)]
mod scratch;
mod slots;
mod starts;

pub use crew::{Crew, Member};
pub use dir::{DirFacts, LockDir, lock_dir_path, unsafe_reason};
pub use error::SlotError;
pub use fifo::Ticket;
pub use held::{HELD_SLOTS_ENV, is_held, mark_held};
pub use key::slot_key;
pub use order::{Release, SLOT_ORDER, SlotPlan, SlotStep, Waiter};
pub use reserve::{Reservation, reserved_total};
pub use slots::{SlotGuard, claim_slot, live_holders};
pub use starts::{
    StartLimits, StartRate, admit_start, parse_start_rate, record_start,
};
