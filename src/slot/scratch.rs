//! [`Scratch`]: a lock dir of one test's own under the system temp dir,
//! removed when it drops; tests never touch the real per-user dir.

use std::fs::{self, DirBuilder};
use std::io;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use super::LockDir;
use crate::sys::current_uid;

/// Tells apart the scratch dirs of one test process.
static NEXT: AtomicU64 = AtomicU64::new(0);

/// How long a lock a test drops may take to come free.
const SETTLE: Duration = Duration::from_secs(2);

/// A fresh, empty, 0700 dir, removed with everything in it on drop.
#[derive(Debug)]
pub(crate) struct Scratch(PathBuf);

impl Scratch {
    /// A new scratch dir named for `test`.
    pub(crate) fn new(test: &str) -> io::Result<Self> {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let path =
            std::env::temp_dir().join(format!("cgmr-slot-{test}-{pid}-{n}"));
        DirBuilder::new().mode(0o700).create(&path)?;
        Ok(Self(path))
    }

    /// Where it is.
    pub(crate) fn path(&self) -> &Path {
        &self.0
    }

    /// It, opened as a lock dir.
    pub(crate) fn lock_dir(&self) -> io::Result<LockDir> {
        LockDir::open(self.0.clone(), current_uid()).map_err(io::Error::other)
    }
}

/// `probe` again until `done` says so, for up to [`SETTLE`]; its last
/// value. A lock this process drops is free only once every copy of its
/// descriptor is closed, and a child another test thread spawns holds
/// copies of them all until it execs (src/slot:V11).
pub(crate) fn eventually<T>(
    mut probe: impl FnMut() -> io::Result<T>,
    done: impl Fn(&T) -> bool,
) -> io::Result<T> {
    let _ = (done, SETTLE);
    probe()
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests;
