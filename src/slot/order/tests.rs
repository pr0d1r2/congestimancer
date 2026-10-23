use std::io;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{Release, SLOT_ORDER, SlotPlan, SlotStep, Waiter};
use crate::slot::scratch::{Scratch, eventually};
use crate::slot::{LockDir, live_holders, reserved_total};
use crate::units::{Bytes, Count, Name, parse_name};

fn key(s: &str) -> io::Result<Name> {
    parse_name(s).map_err(|e| io::Error::other(e.to_string()))
}

fn at(secs: u64) -> SystemTime {
    UNIX_EPOCH
        .checked_add(Duration::from_secs(secs))
        .unwrap_or(UNIX_EPOCH)
}

fn one_slot() -> io::Result<SlotPlan> {
    let mut plan = SlotPlan::new(key("make")?);
    plan.slots = Some(Count(1));
    Ok(plan)
}

fn waiter(dir: &LockDir, plan: &SlotPlan, marker: Option<&str>) -> Waiter {
    Waiter::new(dir.clone(), plan.clone(), marker)
}

fn go(w: &mut Waiter, secs: u64) -> io::Result<Option<Release>> {
    w.try_release(at(secs)).map_err(io::Error::other)
}

#[test]
fn slot_steps_run_fifo_slot_starts_reserve() {
    assert_eq!(
        SLOT_ORDER,
        [
            SlotStep::Fifo,
            SlotStep::Slot,
            SlotStep::Starts,
            SlotStep::Reserve
        ]
    );
}

#[test]
fn a_plan_asking_nothing_releases_at_once() -> io::Result<()> {
    let scratch = Scratch::new("order-none")?;
    let mut w = waiter(&scratch.lock_dir()?, &SlotPlan::new(key("x")?), None);
    assert_eq!(w.steps(), Vec::new());
    let release = go(&mut w, 0)?;
    assert!(release.is_some_and(|r| r.slot().is_none()));
    Ok(())
}

#[test]
fn every_step_asked_for_runs_in_order() -> io::Result<()> {
    let scratch = Scratch::new("order-steps")?;
    let mut plan = one_slot()?;
    plan.fifo = true;
    plan.limits.spacing = Some(Duration::from_secs(1));
    plan.needs_mem = Some(Bytes(1));
    let w = waiter(&scratch.lock_dir()?, &plan, None);
    assert_eq!(w.steps(), SLOT_ORDER.to_vec());
    Ok(())
}

#[test]
fn a_slot_is_held_until_the_release_drops() -> io::Result<()> {
    let scratch = Scratch::new("order-slot")?;
    let dir = scratch.lock_dir()?;
    let plan = one_slot()?;
    let first = go(&mut waiter(&dir, &plan, Some("cargo")), 0)?;
    assert_eq!(
        first.as_ref().and_then(Release::held_marker),
        Some("cargo,make")
    );
    let mut second = waiter(&dir, &plan, None);
    assert!(go(&mut second, 0)?.is_none());
    drop(first);
    let again = eventually(|| go(&mut second, 0), Option::is_some)?;
    assert_eq!(again.and_then(|r| r.slot().map(|s| s.index())), Some(0));
    Ok(())
}

#[test]
fn a_key_an_ancestor_holds_skips_fifo_and_slot() -> io::Result<()> {
    let scratch = Scratch::new("order-held")?;
    let dir = scratch.lock_dir()?;
    let mut plan = one_slot()?;
    plan.fifo = true;
    let _parent = go(&mut waiter(&dir, &plan, None), 0)?;
    let mut child = waiter(&dir, &plan, Some("make"));
    assert_eq!(child.steps(), Vec::new());
    let release = go(&mut child, 0)?;
    assert!(
        release
            .is_some_and(|r| r.slot().is_none() && r.held_marker().is_none())
    );
    Ok(())
}

#[test]
fn fifo_waiters_go_in_arrival_order() -> io::Result<()> {
    let scratch = Scratch::new("order-fifo")?;
    let dir = scratch.lock_dir()?;
    let mut plan = one_slot()?;
    plan.fifo = true;
    let mut a = waiter(&dir, &plan, None);
    let mut b = waiter(&dir, &plan, None);
    let mut c = waiter(&dir, &plan, None);
    let held = go(&mut a, 0)?;
    assert!(held.is_some());
    assert!(go(&mut b, 0)?.is_none());
    assert!(go(&mut c, 0)?.is_none());
    drop(held);
    assert!(go(&mut c, 0)?.is_none());
    assert!(eventually(|| go(&mut b, 0), Option::is_some)?.is_some());
    Ok(())
}

#[test]
fn a_start_refused_lets_go_of_the_slot() -> io::Result<()> {
    let scratch = Scratch::new("order-spacing")?;
    let dir = scratch.lock_dir()?;
    let mut plan = one_slot()?;
    plan.limits.spacing = Some(Duration::from_secs(3600));
    drop(go(&mut waiter(&dir, &plan, None), 100)?);
    assert!(go(&mut waiter(&dir, &plan, None), 200)?.is_none());
    let k = key("make")?;
    let holders = || live_holders(&dir, &k, Count(1)).map_err(io::Error::other);
    assert_eq!(eventually(holders, |n| *n == 0)?, 0);
    Ok(())
}

#[test]
fn a_release_holds_its_memory_reservation() -> io::Result<()> {
    let scratch = Scratch::new("order-mem")?;
    let dir = scratch.lock_dir()?;
    let mut plan = SlotPlan::new(key("make")?);
    plan.needs_mem = Some(Bytes(5000));
    let release = go(&mut waiter(&dir, &plan, None), 0)?;
    let reserved = release.as_ref().and_then(Release::reservation);
    assert_eq!(reserved.map(|r| r.bytes()), Some(Bytes(5000)));
    assert_eq!(reserved_total(&dir).map_err(io::Error::other)?, Bytes(5000));
    Ok(())
}

#[test]
fn alone_takes_the_slot_only_for_the_first_job_of_a_key() -> io::Result<()> {
    let scratch = Scratch::new("order-alone")?;
    let dir = scratch.lock_dir()?;
    let mut plan = one_slot()?;
    plan.alone = true;
    plan.needs_mem = Some(Bytes(7));
    let mut first = waiter(&dir, &plan, None);
    let release = first.try_alone(at(0)).map_err(io::Error::other)?;
    assert!(release.as_ref().is_some_and(|r| r.slot().is_some()));
    assert_eq!(reserved_total(&dir).map_err(io::Error::other)?, Bytes(7));
    let mut second = waiter(&dir, &plan, None);
    let none = second.try_alone(at(0)).map_err(io::Error::other)?;
    assert!(none.is_none());
    Ok(())
}

#[test]
fn alone_asks_nothing_without_the_switch_or_slots() -> io::Result<()> {
    let scratch = Scratch::new("order-alone-off")?;
    let dir = scratch.lock_dir()?;
    let mut w = waiter(&dir, &one_slot()?, None);
    assert!(w.try_alone(at(0)).map_err(io::Error::other)?.is_none());
    let mut plan = SlotPlan::new(key("make")?);
    plan.alone = true;
    let mut w = waiter(&dir, &plan, None);
    assert!(w.try_alone(at(0)).map_err(io::Error::other)?.is_none());
    Ok(())
}
