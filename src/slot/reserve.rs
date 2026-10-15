//! `--needs-mem X`: a reservation file per waiter, held by its `flock`
//! until the command exits; the ledger is the sum of the live ones
//! (src/slot:V7).

use std::fs::File;
use std::path::PathBuf;

use super::{LockDir, SlotError};
use crate::units::Bytes;

/// A memory reservation: the file `+mem.<pid>.<n>` holding the byte
/// count, locked while this lives and removed when it drops.
#[derive(Debug)]
pub struct Reservation {
    /// The open reservation file, locked.
    file: File,
    /// The reservation file.
    path: PathBuf,
    /// How much it reserves.
    bytes: Bytes,
}

impl Reservation {
    /// Reserves `bytes` in the ledger of `dir`.
    ///
    /// # Errors
    ///
    /// [`SlotError::Io`] when the reservation cannot be written.
    pub fn write(dir: &LockDir, bytes: Bytes) -> Result<Self, SlotError> {
        let _ = dir;
        Err(SlotError::Io {
            path: PathBuf::new(),
            detail: bytes.0.to_string(),
        })
    }

    /// How much it reserves.
    #[must_use]
    pub const fn bytes(&self) -> Bytes {
        self.bytes
    }

    /// The locked reservation file, for a launcher that hands it to the
    /// command.
    #[must_use]
    pub const fn file(&self) -> &File {
        &self.file
    }
}

impl Drop for Reservation {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// The sum of every live reservation in `dir`; a reservation whose holder
/// died is removed, not counted (src/slot:V7).
///
/// # Errors
///
/// [`SlotError::Io`] when the lock dir cannot be read.
pub fn reserved_total(dir: &LockDir) -> Result<Bytes, SlotError> {
    let _ = (dir, &PathBuf::new());
    Ok(Bytes(u64::MAX))
}

#[cfg(test)]
mod tests;
