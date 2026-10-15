//! [`Scratch`]: a lock dir of one test's own under the system temp dir,
//! removed when it drops; tests never touch the real per-user dir.

use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::slot::{LockDir, SlotPlan, Waiter};
use crate::sys::current_uid;
use crate::units::{Bytes, Count, parse_name};

/// Tells apart the scratch dirs of one test process.
static NEXT: AtomicU64 = AtomicU64::new(0);

/// A fresh lock dir, removed with everything in it on drop.
#[derive(Debug)]
pub(super) struct Scratch {
    /// Where it is.
    path: PathBuf,
    /// It, opened.
    dir: LockDir,
}

impl Scratch {
    /// A new scratch lock dir named for `test`.
    pub(super) fn new(test: &str) -> io::Result<Self> {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let name = format!("cgmr-launch-{test}-{pid}-{n}");
        let path = std::env::temp_dir().join(name);
        let dir = LockDir::open(path.clone(), current_uid())
            .map_err(io::Error::other)?;
        Ok(Self { path, dir })
    }

    /// A waiter on key `make` for one slot and, when given, `needs_mem`.
    pub(super) fn waiter(&self, needs_mem: Option<u64>) -> io::Result<Waiter> {
        let key = parse_name("make").map_err(io::Error::other)?;
        let mut plan = SlotPlan::new(key);
        plan.slots = Some(Count(1));
        plan.needs_mem = needs_mem.map(Bytes);
        Ok(Waiter::new(self.dir.clone(), plan, None))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests;
