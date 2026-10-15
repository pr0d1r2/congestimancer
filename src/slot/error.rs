//! [`SlotError`]: why a slot step cannot run.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// Why a slot step cannot run.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SlotError {
    /// The lock dir is not safe to use: a symlink, not a directory, owned
    /// by another user, or open to others. Exit 69 (src/slot:V1).
    UnsafeDir {
        /// The lock dir.
        path: PathBuf,
        /// What is wrong with it.
        reason: String,
    },
    /// The slot key is not a safe name, or none could be made from the
    /// command (src/slot:V3, V10). A usage error, exit 64.
    BadKey {
        /// The key, or the command it was made from, as given.
        input: String,
    },
    /// The lock dir could not be read or written.
    Io {
        /// The path that failed.
        path: PathBuf,
        /// The OS error, in words.
        detail: String,
    },
}

impl fmt::Display for SlotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsafeDir { path, reason } => {
                write!(f, "slot dir {}: {reason}; refusing", path.display())
            }
            Self::BadKey { input } => write!(
                f,
                "--slot-key {input:?}: not a safe name (1 to 64 of A-Z a-z \
                 0-9 . _ -)"
            ),
            Self::Io { path, detail } => {
                write!(f, "slot dir {}: {detail}", path.display())
            }
        }
    }
}

impl std::error::Error for SlotError {}

impl SlotError {
    /// `err` on `path` as [`SlotError::Io`].
    pub(crate) fn io(path: &Path, err: &io::Error) -> Self {
        Self::Io {
            path: path.to_owned(),
            detail: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests;
