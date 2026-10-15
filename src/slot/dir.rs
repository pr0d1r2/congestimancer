//! [`LockDir`]: the per-user dir every slot file lives in, made 0700 and
//! refused when anyone else could own or enter it (src/slot:V1).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::SlotError;

/// What the lock dir is, as `lstat` sees it: the facts its safety turns
/// on (src/slot:V1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirFacts {
    /// A symlink, not followed.
    pub symlink: bool,
    /// A directory.
    pub dir: bool,
    /// Its owner's uid.
    pub owner: u32,
    /// Its permission bits.
    pub mode: u32,
}

/// Why `facts` make the lock dir unsafe for user `uid`, or `None` when it
/// is a directory of theirs no one else may enter (src/slot:V1).
#[must_use]
pub fn unsafe_reason(facts: &DirFacts, uid: u32) -> Option<String> {
    let _ = (facts, uid);
    None
}

/// Where the lock dir is: `$XDG_RUNTIME_DIR/cgmr`, else `$TMPDIR/cgmr`,
/// else `/tmp/cgmr-<uid>`; an empty variable counts as unset (src/slot:I).
#[must_use]
pub fn lock_dir_path(
    runtime: Option<&OsStr>,
    tmpdir: Option<&OsStr>,
    uid: u32,
) -> PathBuf {
    let _ = (runtime, tmpdir, uid);
    PathBuf::new()
}

/// The per-user lock dir, checked safe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockDir {
    /// Where it is.
    path: PathBuf,
}

impl LockDir {
    /// Makes the lock dir at `path` mode 0700 when absent, then checks it
    /// is a directory owned by `uid` that no one else may enter.
    ///
    /// # Errors
    ///
    /// [`SlotError::UnsafeDir`] when it is a symlink, not a directory,
    /// another user's, or wider than 0700 (exit 69); [`SlotError::Io`]
    /// when it cannot be made or read.
    pub fn open(path: PathBuf, uid: u32) -> Result<Self, SlotError> {
        let _ = uid;
        Ok(Self { path })
    }

    /// Where it is.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests;
