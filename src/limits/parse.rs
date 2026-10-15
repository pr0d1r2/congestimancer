//! Linux limits text and the share of a maximum in use (src/limits:T1,
//! V1, V3).

use super::LimitsError;
use crate::units::Percent;

/// `/proc/sys/fs/file-nr`: file handles allocated, allocated but unused,
/// and the system maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct FileNr {
    /// File handles allocated.
    pub allocated: u64,
    /// Allocated handles not in use (always 0 since Linux 2.6).
    pub unused: u64,
    /// The system maximum, `fs.file-max`.
    pub max: u64,
}

impl FileNr {
    /// File handles open: allocated less unused.
    #[must_use]
    pub const fn open(self) -> u64 {
        let _ = self;
        0
    }
}

/// `/proc/sys/fs/file-nr`: three whole numbers, such as
/// `3008 0 9223372036854775807`.
///
/// # Errors
///
/// [`LimitsError::Malformed`] unless the text is exactly three whole
/// numbers.
pub fn parse_file_nr(text: &str) -> Result<FileNr, LimitsError> {
    Err(LimitsError::malformed("file-nr", text))
}

/// `used` as a share of `max`, rounded down to a hundredth of a percent
/// and capped at 100%; `None` when `max` is 0 (src/limits:V3).
#[must_use]
pub fn limit_share(used: u64, max: u64) -> Option<Percent> {
    let _ = (used, max);
    None
}

#[cfg(test)]
mod tests;
