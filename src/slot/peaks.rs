//! `--needs-mem auto`: the memory peak each `--mem-key` was seen to need
//! (src/slot:V13).

use super::{LockDir, SlotError};
use crate::units::{Bytes, Name};

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
    _dir: &LockDir,
    _key: &Name,
) -> Result<Option<Bytes>, SlotError> {
    Ok(None)
}

/// Every key with a learned peak, by key.
///
/// # Errors
///
/// [`SlotError::Io`] when the lock dir cannot be read.
pub fn learned_peaks(_dir: &LockDir) -> Result<Vec<Learned>, SlotError> {
    Ok(Vec::new())
}

/// Records that a run of `key` peaked at `observed`; returns the peak
/// now kept.
///
/// # Errors
///
/// [`SlotError::Io`] when the peak file cannot be written.
pub fn record_peak(
    _dir: &LockDir,
    _key: &Name,
    _observed: Bytes,
) -> Result<Bytes, SlotError> {
    Ok(Bytes(0))
}

/// What to reserve for a job that peaked at `peak`.
#[must_use]
pub const fn reserve_for(peak: Bytes) -> Bytes {
    peak
}

#[cfg(test)]
mod tests;
