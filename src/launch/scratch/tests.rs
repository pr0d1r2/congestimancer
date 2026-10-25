use std::io;

use super::{Scratch, eventually};
use crate::slot::SlotStep;

#[test]
fn makes_a_fresh_dir_and_removes_it_on_drop() -> io::Result<()> {
    let scratch = Scratch::new("own")?;
    let other = Scratch::new("own")?;
    let path = scratch.path.clone();
    assert!(path.is_dir());
    assert_ne!(path, other.path);
    drop(scratch);
    assert!(!path.exists());
    Ok(())
}

#[test]
fn waiter_asks_one_slot_and_the_memory_given() -> io::Result<()> {
    let scratch = Scratch::new("plan")?;
    let plain = scratch.waiter(None)?.steps();
    assert_eq!(plain, [SlotStep::Slot]);
    let reserving = scratch.waiter(Some(1))?.steps();
    assert_eq!(reserving, [SlotStep::Slot, SlotStep::Reserve]);
    Ok(())
}

#[test]
fn sharing_asks_a_slot_and_no_memory() -> io::Result<()> {
    let scratch = Scratch::new("sharing")?;
    assert_eq!(scratch.sharing(2)?.steps(), [SlotStep::Slot]);
    Ok(())
}

#[test]
fn eventually_probes_until_done_and_passes_errors_on() {
    let mut calls = 0;
    let third = eventually(
        || {
            calls += 1;
            Ok(calls)
        },
        |n| *n == 3,
    );
    assert_eq!(third.ok(), Some(3));
    let failed = eventually(|| Err::<u8, _>(io::Error::other("x")), |_| true);
    assert!(failed.is_err());
}
