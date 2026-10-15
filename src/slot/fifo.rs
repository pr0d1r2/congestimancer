//! `--fifo`: waiters take numbered tickets; only the lowest live ticket
//! may claim a free slot, and tickets whose holder died are skipped
//! (src/slot:V4).

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use super::file::{
    lock_wait, open_lock, publish, read_all, reap_if_dead, rewrite, suffixes,
};
use super::{LockDir, SlotError};
use crate::units::Name;

/// A place in the queue for a key: the file `K+ticket.<seq>`, locked
/// while this lives and removed when it drops (src/slot:V4).
#[derive(Debug)]
pub struct Ticket {
    /// The open ticket file, locked; held, never read.
    _held: File,
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
        let counter = dir.path().join(format!("{key}+seq"));
        let seq =
            next_seq(&counter).map_err(|e| SlotError::io(&counter, &e))?;
        let name = format!("{key}+ticket.{seq}");
        let file = publish(dir.path(), &name, "")
            .map_err(|e| SlotError::io(dir.path(), &e))?;
        Ok(Self {
            _held: file,
            path: dir.path().join(name),
            dir: dir.path().to_owned(),
            key: key.clone(),
            seq,
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
        let prefix = format!("{}+ticket.", self.key);
        let seqs = suffixes(&self.dir, &prefix)
            .map_err(|e| SlotError::io(&self.dir, &e))?;
        let lower = seqs
            .iter()
            .filter_map(|s| s.parse::<u64>().ok())
            .filter(|&seq| seq < self.seq);
        for seq in lower {
            let path = self.dir.join(format!("{prefix}{seq}"));
            if !reap_if_dead(&path).map_err(|e| SlotError::io(&path, &e))? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl Drop for Ticket {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// The next ticket number from the counter at `path`: tickets start at 1,
/// and the counter only grows, under its `flock`.
fn next_seq(path: &Path) -> io::Result<u64> {
    let mut file = open_lock(path)?;
    lock_wait(&file)?;
    let last = read_all(&mut file)?.trim().parse::<u64>().unwrap_or(0);
    let seq = last.saturating_add(1);
    rewrite(&mut file, &seq.to_string())?;
    Ok(seq)
}

#[cfg(test)]
mod tests;
