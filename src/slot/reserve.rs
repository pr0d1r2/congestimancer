//! `--needs-mem X`: a reservation file per waiter, held by its `flock`
//! until the command exits; the ledger is the sum of the live ones
//! (src/slot:V7).

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::file::{publish, reap_if_dead, suffixes};
use super::{LockDir, SlotError};
use crate::units::Bytes;

/// The name every reservation file starts with.
const PREFIX: &str = "+mem.";

/// Tells apart the reservations of this process.
static NEXT: AtomicU64 = AtomicU64::new(0);

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
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let name = format!("{PREFIX}{}.{n}", std::process::id());
        let file = publish(dir.path(), &name, &bytes.0.to_string())
            .map_err(|e| SlotError::io(dir.path(), &e))?;
        let path = dir.path().join(name);
        Ok(Self { file, path, bytes })
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
        let _ = fs::remove_file(&self.path);
    }
}

/// The sum of every live reservation in `dir`; a reservation whose holder
/// died is removed, not counted (src/slot:V7).
///
/// # Errors
///
/// [`SlotError::Io`] when the lock dir cannot be read.
pub fn reserved_total(dir: &LockDir) -> Result<Bytes, SlotError> {
    let names = suffixes(dir.path(), PREFIX)
        .map_err(|e| SlotError::io(dir.path(), &e))?;
    let mut total: u64 = 0;
    for name in names {
        let path = dir.path().join(format!("{PREFIX}{name}"));
        let live = live_bytes(&path).map_err(|e| SlotError::io(&path, &e))?;
        total = total.saturating_add(live);
    }
    Ok(Bytes(total))
}

/// The bytes the reservation at `path` holds, or 0 when its holder is
/// gone (the file is then removed) or it does not parse.
fn live_bytes(path: &Path) -> io::Result<u64> {
    if reap_if_dead(path)? {
        return Ok(0);
    }
    match fs::read_to_string(path) {
        Ok(text) => Ok(text.trim().parse().unwrap_or(0)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(0),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests;
