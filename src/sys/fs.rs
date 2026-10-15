//! Files and the process through `rustix`: [`statvfs`] for free space and
//! inodes, `flock` locks for slots, and the process nice value
//! (src/sys:T4, V2).

use std::fs::File;
use std::io;
use std::path::Path;

use rustix::fs::{FlockOperation, StatVfs};

/// Space and inodes of the filesystem holding a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct FsStats {
    /// The unit `blocks*` count in, in bytes.
    pub fragment_size: u64,
    /// Blocks in the filesystem.
    pub blocks: u64,
    /// Free blocks, including those reserved for root.
    pub blocks_free: u64,
    /// Free blocks an unprivileged user may take.
    pub blocks_available: u64,
    /// Inodes in the filesystem; 0 when it does not count them.
    pub files: u64,
    /// Free inodes, including those reserved for root.
    pub files_free: u64,
    /// Free inodes an unprivileged user may take.
    pub files_available: u64,
}

impl FsStats {
    /// The filesystem size in bytes, saturating.
    #[must_use]
    pub const fn bytes_total(&self) -> u64 {
        self.blocks.saturating_mul(self.fragment_size)
    }

    /// Bytes an unprivileged user may still write, saturating.
    #[must_use]
    pub const fn bytes_available(&self) -> u64 {
        self.blocks_available.saturating_mul(self.fragment_size)
    }
}

impl From<StatVfs> for FsStats {
    fn from(raw: StatVfs) -> Self {
        Self {
            fragment_size: raw.f_frsize,
            blocks: raw.f_blocks,
            blocks_free: raw.f_bfree,
            blocks_available: raw.f_bavail,
            files: raw.f_files,
            files_free: raw.f_ffree,
            files_available: raw.f_favail,
        }
    }
}

/// Space and inodes of the filesystem holding `path`.
///
/// # Errors
///
/// The OS error, such as [`io::ErrorKind::NotFound`] for a path that does
/// not exist.
pub fn statvfs(path: &Path) -> io::Result<FsStats> {
    Ok(rustix::fs::statvfs(path)?.into())
}

/// Takes an exclusive `flock` on `file` if no other open file holds one:
/// `Ok(false)` when another does, without waiting.
///
/// # Errors
///
/// The OS error for anything but "already locked".
pub fn try_lock_exclusive(file: &File) -> io::Result<bool> {
    match rustix::fs::flock(file, FlockOperation::NonBlockingLockExclusive) {
        Ok(()) => Ok(true),
        Err(rustix::io::Errno::WOULDBLOCK) => Ok(false),
        Err(err) => Err(err.into()),
    }
}

/// Releases this open file's `flock`.
///
/// # Errors
///
/// The OS error.
pub fn unlock(file: &File) -> io::Result<()> {
    Ok(rustix::fs::flock(file, FlockOperation::Unlock)?)
}

/// Lets `file`'s descriptor survive `exec` by clearing its
/// `FD_CLOEXEC` flag, so a slot lock passes to the governed command
/// (src/sys:T19).
///
/// # Errors
///
/// The OS error.
pub const fn set_inheritable(file: &File) -> io::Result<()> {
    let _ = file;
    Ok(())
}

/// This process's nice value.
///
/// # Errors
///
/// The OS error.
pub fn nice() -> io::Result<i32> {
    Ok(rustix::process::getpriority_process(None)?)
}

/// Sets this process's nice value; it carries across `exec`.
///
/// # Errors
///
/// [`io::ErrorKind::PermissionDenied`] when lowering it without privilege.
pub fn set_nice(value: i32) -> io::Result<()> {
    Ok(rustix::process::setpriority_process(None, value)?)
}

#[cfg(test)]
mod tests;
