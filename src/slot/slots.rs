//! `--slots N`: at most N live holders of a key, each holding the `flock`
//! on one of the files `K.0` to `K.<N-1>` (src/slot:V2).

use std::fs::File;
use std::path::PathBuf;

use super::file::open_lock;
use super::{LockDir, SlotError};
use crate::sys::try_lock_exclusive;
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
    for index in 0..slots.0 {
        if let Some(file) = try_slot(dir, key, index)? {
            return Ok(Some(SlotGuard { file, index }));
        }
    }
    Ok(None)
}

/// The file of slot `index` of `key`.
fn slot_path(dir: &LockDir, key: &Name, index: u64) -> PathBuf {
    dir.path().join(format!("{key}.{index}"))
}

/// Slot `index` of `key`, opened and locked, or `None` when it is held.
fn try_slot(
    dir: &LockDir,
    key: &Name,
    index: u64,
) -> Result<Option<File>, SlotError> {
    let path = slot_path(dir, key, index);
    let file = open_lock(&path).map_err(|e| SlotError::io(&path, &e))?;
    let free =
        try_lock_exclusive(&file).map_err(|e| SlotError::io(&path, &e))?;
    Ok(free.then_some(file))
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
    let mut held: u64 = 0;
    for index in 0..slots.0 {
        if try_slot(dir, key, index)?.is_none() {
            held = held.saturating_add(1);
        }
    }
    Ok(held)
}

#[cfg(test)]
mod tests;
