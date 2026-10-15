use std::io;

use std::thread;
use std::time::Duration;

use super::{Scratch, eventually};
use crate::slot::file::open_lock;
use crate::sys::try_lock_exclusive;

#[test]
fn a_scratch_dir_is_fresh_and_goes_on_drop() -> io::Result<()> {
    let first = Scratch::new("scratch")?;
    let second = Scratch::new("scratch")?;
    assert_ne!(first.path(), second.path());
    let path = first.path().to_owned();
    assert!(path.is_dir());
    drop(first);
    assert!(!path.exists());
    Ok(())
}

#[test]
fn a_lock_a_spawn_still_holds_comes_free_eventually() -> io::Result<()> {
    let scratch = Scratch::new("eventually")?;
    let path = scratch.path().join("k.0");
    let held = open_lock(&path)?;
    assert!(try_lock_exclusive(&held)?);
    // A copy of the descriptor, as a child mid-spawn has, closed later.
    let copy = held.try_clone()?;
    drop(held);
    let closer = thread::spawn(move || {
        thread::sleep(Duration::from_millis(20));
        drop(copy);
    });
    let free = eventually(|| try_lock_exclusive(&open_lock(&path)?), |f| *f);
    closer.join().map_err(|_| io::Error::other("closer"))?;
    assert!(free?, "the lock never came free");
    Ok(())
}
