//! `--needs-mem auto`: the memory peak each `--mem-key` was seen to need,
//! kept as `+peak.<key>` in the lock dir under its `flock`, so the next
//! start with that key can reserve it (src/slot:V13).

use std::io;
use std::path::Path;

use super::file::{lock_wait, open_lock, read_all, rewrite, suffixes};
use super::{LockDir, SlotError};
use crate::units::{Bytes, Name, parse_name};

/// The name every peak file starts with; `+` is no part of a key, so a
/// slot file can never be taken for one.
const PREFIX: &str = "+peak.";

/// How much of an older peak a smaller one leaves: three quarters, so a
/// lucky small run does not make the next start under-reserve.
const KEEP_NUMERATOR: u64 = 3;

/// The denominator of [`KEEP_NUMERATOR`].
const KEEP_DENOMINATOR: u64 = 4;

/// The margin reserved over a learned peak, as a divisor: a tenth.
const MARGIN_DIVISOR: u64 = 10;

/// A key and the peak learned for it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Learned {
    /// The `--mem-key`.
    pub key: String,
    /// The peak kept for it.
    pub peak: Bytes,
}

/// The peak learned for `key`, if a run of it was ever measured.
///
/// # Errors
///
/// [`SlotError::Io`] when the lock dir cannot be read.
pub fn learned_peak(
    dir: &LockDir,
    key: &Name,
) -> Result<Option<Bytes>, SlotError> {
    let path = dir.path().join(format!("{PREFIX}{key}"));
    let mut file = match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(SlotError::io(&path, &err)),
    };
    lock_wait(&file).map_err(|err| SlotError::io(&path, &err))?;
    let text = read_all(&mut file).map_err(|err| SlotError::io(&path, &err))?;
    Ok(parse(&text))
}

/// Every key with a learned peak, by key.
///
/// # Errors
///
/// [`SlotError::Io`] when the lock dir cannot be read.
pub fn learned_peaks(dir: &LockDir) -> Result<Vec<Learned>, SlotError> {
    let mut names = suffixes(dir.path(), PREFIX)
        .map_err(|e| SlotError::io(dir.path(), &e))?;
    names.sort_unstable();
    let mut found = Vec::new();
    for name in names {
        let Ok(key) = parse_name(&name) else {
            continue;
        };
        if let Some(peak) = learned_peak(dir, &key)? {
            found.push(Learned { key: name, peak });
        }
    }
    Ok(found)
}

/// Records that a run of `key` peaked at `observed`: the larger of it and
/// three quarters of the peak kept so far. Nothing is recorded for
/// `Bytes(0)`, a run no sample caught. Returns the peak now kept.
///
/// # Errors
///
/// [`SlotError::Io`] when the peak file cannot be read or written.
pub fn record_peak(
    dir: &LockDir,
    key: &Name,
    observed: Bytes,
) -> Result<Bytes, SlotError> {
    let path = dir.path().join(format!("{PREFIX}{key}"));
    if observed.0 == 0 {
        return Ok(learned_peak(dir, key)?.unwrap_or_default());
    }
    keep(&path, observed).map_err(|e| SlotError::io(&path, &e))
}

/// What to reserve for a job that peaked at `peak`: the peak and a tenth
/// more, saturating (src/slot:V13).
#[must_use]
pub const fn reserve_for(peak: Bytes) -> Bytes {
    Bytes(peak.0.saturating_add(peak.0 / MARGIN_DIVISOR))
}

/// [`record_peak`] on the file at `path`, under its `flock`.
fn keep(path: &Path, observed: Bytes) -> io::Result<Bytes> {
    let mut file = open_lock(path)?;
    lock_wait(&file)?;
    let before = parse(&read_all(&mut file)?).unwrap_or_default();
    let decayed = before.0.saturating_mul(KEEP_NUMERATOR) / KEEP_DENOMINATOR;
    let kept = Bytes(observed.0.max(decayed));
    rewrite(&mut file, &kept.0.to_string())?;
    Ok(kept)
}

/// The byte count in a peak file; `None` for anything else or zero.
fn parse(text: &str) -> Option<Bytes> {
    text.trim().parse().ok().filter(|&n| n > 0).map(Bytes)
}

#[cfg(test)]
mod tests;
