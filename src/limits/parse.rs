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
        self.allocated.saturating_sub(self.unused)
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
    let bad = || LimitsError::malformed("file-nr", text);
    let mut fields = text.split_whitespace().map(str::parse::<u64>);
    let mut next = || fields.next().and_then(Result::ok).ok_or_else(bad);
    let nr = FileNr {
        allocated: next()?,
        unused: next()?,
        max: next()?,
    };
    match fields.next() {
        None => Ok(nr),
        Some(_) => Err(bad()),
    }
}

/// `used` as a share of `max`, rounded down to a hundredth of a percent
/// and capped at 100%; `None` when `max` is 0 (src/limits:V3).
#[must_use]
pub fn limit_share(used: u64, max: u64) -> Option<Percent> {
    let hundredths = u128::from(used)
        .saturating_mul(u128::from(Percent::FULL.hundredths()))
        .checked_div(u128::from(max))?;
    let capped = hundredths.min(u128::from(Percent::FULL.hundredths()));
    u16::try_from(capped)
        .ok()
        .and_then(Percent::from_hundredths)
}

#[cfg(test)]
mod tests;
