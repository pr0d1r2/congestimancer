//! [`IoError`]: why an IO file did not parse, or no device was found.

use std::fmt;
use std::path::PathBuf;

use crate::units::UnitError;

/// Why Linux IO text did not parse, why a path has no block device, or
/// why an `--io-device` was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum IoError {
    /// A line is not in the form its file takes.
    Malformed {
        /// The file, such as `/proc/diskstats`.
        file: &'static str,
        /// The line as read, trimmed.
        line: String,
    },
    /// No mount holds the path.
    NoMount {
        /// The path asked about.
        path: PathBuf,
    },
    /// The path's filesystem has no block device under it: tmpfs, NFS,
    /// FUSE (src/io:V3).
    NotBlock {
        /// The path asked about.
        path: PathBuf,
        /// Its filesystem type, such as `tmpfs`.
        fs_type: String,
    },
    /// No block device of this name is counted.
    UnknownDevice {
        /// The device name, such as `sdz`.
        name: String,
    },
    /// An `--io-device` that is not a name (src/io:V10).
    Device(UnitError),
}

impl IoError {
    /// A [`IoError::Malformed`] for `line` of `file`.
    pub(crate) fn malformed(file: &'static str, line: &str) -> Self {
        Self::Malformed {
            file,
            line: line.trim().to_owned(),
        }
    }
}

/// The tail of [`IoError::NotBlock`]: what it lacks and the way out.
const NO_BLOCK: &str = "not a block device; name one with --io-device";

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { file, line } => {
                write!(f, "malformed {file} line: `{line}`")
            }
            Self::NoMount { path } => {
                write!(f, "no mount holds `{}`", path.display())
            }
            Self::NotBlock { path, fs_type } => {
                write!(f, "`{}` is on {fs_type}, {NO_BLOCK}", path.display())
            }
            Self::UnknownDevice { name } => {
                write!(f, "no block device `{name}`")
            }
            Self::Device(err) => write!(f, "--io-device: {err}"),
        }
    }
}

impl std::error::Error for IoError {}

#[cfg(test)]
mod tests;
