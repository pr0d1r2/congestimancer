use std::io;

use super::{claim_slot, live_holders};
use crate::slot::scratch::Scratch;
use crate::units::{Count, Name, parse_name};

fn key(s: &str) -> io::Result<Name> {
    parse_name(s).map_err(|e| io::Error::other(e.to_string()))
}

fn index(scratch: &Scratch, k: &Name, n: u64) -> io::Result<Option<u64>> {
    let dir = scratch.lock_dir()?;
    let guard = claim_slot(&dir, k, Count(n)).map_err(io::Error::other)?;
    Ok(guard.as_ref().map(super::SlotGuard::index))
}

#[test]
fn slots_fill_in_order_and_refuse_past_n() -> io::Result<()> {
    let scratch = Scratch::new("slots-fill")?;
    let dir = scratch.lock_dir()?;
    let k = key("make")?;
    let claim = || claim_slot(&dir, &k, Count(2)).map_err(io::Error::other);
    let first = claim()?;
    let second = claim()?;
    assert_eq!(first.as_ref().map(super::SlotGuard::index), Some(0));
    assert_eq!(second.as_ref().map(super::SlotGuard::index), Some(1));
    assert!(claim()?.is_none());
    drop(first);
    assert_eq!(claim()?.map(|g| g.index()), Some(0));
    Ok(())
}

#[test]
fn a_slot_is_a_file_named_key_dot_index() -> io::Result<()> {
    let scratch = Scratch::new("slots-name")?;
    let dir = scratch.lock_dir()?;
    let guard = claim_slot(&dir, &key("cargo")?, Count(1));
    assert!(guard.is_ok_and(|g| g.is_some()));
    assert!(scratch.path().join("cargo.0").is_file());
    Ok(())
}

#[test]
fn zero_slots_are_never_free() -> io::Result<()> {
    let scratch = Scratch::new("slots-zero")?;
    assert_eq!(index(&scratch, &key("x")?, 0)?, None);
    Ok(())
}

#[test]
fn keys_do_not_share_slots() -> io::Result<()> {
    let scratch = Scratch::new("slots-keys")?;
    assert_eq!(index(&scratch, &key("a")?, 1)?, Some(0));
    assert_eq!(index(&scratch, &key("b")?, 1)?, Some(0));
    Ok(())
}

#[test]
fn live_holders_counts_held_slots() -> io::Result<()> {
    let scratch = Scratch::new("slots-live")?;
    let dir = scratch.lock_dir()?;
    let k = key("make")?;
    let count = || live_holders(&dir, &k, Count(3)).map_err(io::Error::other);
    assert_eq!(count()?, 0);
    let held = claim_slot(&dir, &k, Count(3)).map_err(io::Error::other)?;
    assert_eq!(count()?, 1);
    drop(held);
    assert_eq!(count()?, 0);
    Ok(())
}

#[test]
fn a_slot_held_by_another_thread_is_not_free() -> io::Result<()> {
    let scratch = Scratch::new("slots-thread")?;
    let dir = scratch.lock_dir()?;
    let k = key("make")?;
    let held = std::thread::scope(|s| {
        s.spawn(|| claim_slot(&dir, &k, Count(1))).join()
    })
    .map_err(|_| io::Error::other("thread"))?
    .map_err(io::Error::other)?;
    assert!(held.is_some());
    let again = claim_slot(&dir, &k, Count(1)).map_err(io::Error::other)?;
    assert!(again.is_none());
    Ok(())
}
