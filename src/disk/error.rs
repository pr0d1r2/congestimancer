//! [`DiskError`]: why a `--disk-path` cannot be gated on.

use std::fmt;
use std::path::PathBuf;

/// Why a `--disk-path` cannot be gated on: a usage error, exit 64
/// (src/disk:V3).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiskError {
    /// The path does not exist.
    Missing {
        /// The path as given.
        path: PathBuf,
    },
}

impl fmt::Display for DiskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { path } => {
                write!(f, "--disk-path {}: no such path", path.display())
            }
        }
    }
}

impl std::error::Error for DiskError {}

#[cfg(test)]
mod tests;
