use std::io;

use super::Scratch;
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
