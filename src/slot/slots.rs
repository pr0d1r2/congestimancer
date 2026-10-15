//! `--slots N`: at most N live holders of a key, each holding the `flock`
//! on one of the files `K.0` to `K.<N-1>` (src/slot:V2).

use std::fs::File;

use super::{LockDir, SlotError};
use crate::units::{Count, Name};

/// A held slot: the `flock` on `K.<i>` lasts as long as this guard, and
/// the kernel drops it when the holder dies (src/slot:V2).
#[derive(Debug)]
pub struct SlotGuard {
    /// The open slot file, locked.
    file: File,
    /// `i` in `K.<i>`.
    index: u64,
}

impl SlotGuard {
    /// `i` in `K.<i>`.
    #[must_use]
    pub const fn index(&self) -> u64 {
        self.index
    }

    /// The locked slot file, for a launcher that hands it to the command.
    #[must_use]
    pub const fn file(&self) -> &File {
        &self.file
    }
}

/// Takes the first free one of `slots` slots of `key`, or `None` when all
/// are held (src/slot:V2).
///
/// # Errors
///
/// [`SlotError::Io`] when a slot file cannot be opened or locked.
pub fn claim_slot(
    dir: &LockDir,
    key: &Name,
    slots: Count,
) -> Result<Option<SlotGuard>, SlotError> {
    let _ = (dir, key, slots);
    Ok(None)
}

/// How many of `slots` slots of `key` are held right now.
///
/// # Errors
///
/// [`SlotError::Io`] when a slot file cannot be opened or locked.
pub fn live_holders(
    dir: &LockDir,
    key: &Name,
    slots: Count,
) -> Result<u64, SlotError> {
    let _ = (dir, key, slots);
    Ok(u64::MAX)
}

#[cfg(test)]
mod tests;
