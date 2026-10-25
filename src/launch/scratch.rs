//! [`Scratch`]: a lock dir of one test's own under the system temp dir,
//! removed when it drops; tests never touch the real per-user dir.

use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::slot::{LockDir, SlotPlan, Waiter};
use crate::sys::current_uid;
use crate::units::{Bytes, Count, parse_name};

/// Tells apart the scratch dirs of one test process.
static NEXT: AtomicU64 = AtomicU64::new(0);

/// How long a lock a test drops may take to come free.
const SETTLE: Duration = Duration::from_secs(2);

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
        self.on(1, needs_mem)
    }

    /// A waiter on key `make`, which `slots` jobs may hold at once.
    pub(super) fn sharing(&self, slots: u64) -> io::Result<Waiter> {
        self.on(slots, None)
    }

    /// A waiter on key `make` for one of `slots` slots and, when given,
    /// `needs_mem`.
    fn on(&self, slots: u64, needs_mem: Option<u64>) -> io::Result<Waiter> {
        let key = parse_name("make").map_err(io::Error::other)?;
        let mut plan = SlotPlan::new(key);
        plan.slots = Some(Count(slots));
        plan.needs_mem = needs_mem.map(Bytes);
        Ok(Waiter::new(self.dir.clone(), plan, None))
    }
}

/// `probe` again until `done` says so, for up to [`SETTLE`]; its last
/// value. A slot this process drops is free only once every copy of its
/// descriptor is closed, and a child another test thread spawns holds
/// copies of them all until it execs (src/slot:V11, src/launch:B2).
pub(super) fn eventually<T>(
    mut probe: impl FnMut() -> io::Result<T>,
    done: impl Fn(&T) -> bool,
) -> io::Result<T> {
    let start = Instant::now();
    loop {
        let value = probe()?;
        if done(&value) || start.elapsed() > SETTLE {
            return Ok(value);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests;
