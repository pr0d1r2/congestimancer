use std::fs;
use std::io;

use super::{Reservation, reserved_total};
use crate::slot::LockDir;
use crate::slot::scratch::Scratch;
use crate::units::Bytes;

fn reserve(dir: &LockDir, n: u64) -> io::Result<Reservation> {
    Reservation::write(dir, Bytes(n)).map_err(io::Error::other)
}

fn total(dir: &LockDir) -> io::Result<u64> {
    reserved_total(dir).map(|b| b.0).map_err(io::Error::other)
}

#[test]
fn an_empty_ledger_reserves_nothing() -> io::Result<()> {
    let scratch = Scratch::new("reserve-empty")?;
    assert_eq!(total(&scratch.lock_dir()?)?, 0);
    Ok(())
}

#[test]
fn live_reservations_add_up_and_go_on_drop() -> io::Result<()> {
    let scratch = Scratch::new("reserve-sum")?;
    let dir = scratch.lock_dir()?;
    let one = reserve(&dir, 1_000)?;
    let two = reserve(&dir, 2_000)?;
    assert_eq!(one.bytes(), Bytes(1_000));
    assert_eq!(total(&dir)?, 3_000);
    drop(one);
    assert_eq!(total(&dir)?, 2_000);
    drop(two);
    assert_eq!(total(&dir)?, 0);
    assert_eq!(fs::read_dir(scratch.path())?.count(), 0);
    Ok(())
}

#[test]
fn a_dead_reservation_is_removed_not_counted() -> io::Result<()> {
    let scratch = Scratch::new("reserve-dead")?;
    let dir = scratch.lock_dir()?;
    let dead = scratch.path().join("+mem.1.0");
    fs::write(&dead, "5000")?;
    let _live = reserve(&dir, 7)?;
    assert_eq!(total(&dir)?, 7);
    assert!(!dead.exists());
    Ok(())
}

#[test]
fn the_sum_saturates() -> io::Result<()> {
    let scratch = Scratch::new("reserve-max")?;
    let dir = scratch.lock_dir()?;
    let _a = reserve(&dir, u64::MAX)?;
    let _b = reserve(&dir, 1)?;
    assert_eq!(total(&dir)?, u64::MAX);
    Ok(())
}
