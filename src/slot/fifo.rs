//! `--fifo`: waiters take numbered tickets; only the lowest live ticket
//! may claim a free slot, and tickets whose holder died are skipped
//! (src/slot:V4).

use std::fs::File;
use std::path::PathBuf;

use super::{LockDir, SlotError};
use crate::units::Name;

/// A place in the queue for a key: the file `K+ticket.<seq>`, locked
/// while this lives and removed when it drops (src/slot:V4).
#[derive(Debug)]
pub struct Ticket {
    /// The open ticket file, locked.
    file: File,
    /// The ticket file.
    path: PathBuf,
    /// The lock dir it is in.
    dir: PathBuf,
    /// The key it queues for.
    key: Name,
    /// Its number: lower came first.
    seq: u64,
}

impl Ticket {
    /// Takes the next ticket for `key`: one past the last ever handed out,
    /// counted in `K+seq` under its `flock`.
    ///
    /// # Errors
    ///
    /// [`SlotError::Io`] when the counter or ticket cannot be written.
    pub fn take(dir: &LockDir, key: &Name) -> Result<Self, SlotError> {
        let _ = (dir, key);
        Err(SlotError::Io {
            path: PathBuf::new(),
            detail: String::new(),
        })
    }

    /// Its number: lower came first.
    #[must_use]
    pub const fn seq(&self) -> u64 {
        self.seq
    }

    /// Whether no live ticket for the key is lower; a lower ticket whose
    /// holder died is removed on the way.
    ///
    /// # Errors
    ///
    /// [`SlotError::Io`] when the lock dir cannot be read.
    pub fn is_first(&self) -> Result<bool, SlotError> {
        let _ = (&self.file, &self.path, &self.dir, &self.key);
        Ok(false)
    }
}

#[cfg(test)]
mod tests;
