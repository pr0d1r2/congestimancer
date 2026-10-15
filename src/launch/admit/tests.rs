use std::io;

use super::Slots;
use crate::launch::scratch::Scratch;
use crate::wait::Admit;

#[test]
fn no_slot_switch_admits_at_once_holding_nothing() {
    let admitted = Slots::none().try_admit();
    assert!(matches!(admitted, Ok(Some(None))), "{admitted:?}");
}

#[test]
fn a_slot_is_held_until_its_release_drops() -> io::Result<()> {
    let scratch = Scratch::new("held")?;
    let mut first = Slots::new(scratch.waiter(None)?);
    let mut second = Slots::new(scratch.waiter(None)?);
    let held = first.try_admit().map_err(io::Error::other)?;
    let Some(Some(release)) = held else {
        unreachable!("the only slot was free: {held:?}");
    };
    assert!(release.slot().is_some());
    let waits = second.try_admit().map_err(io::Error::other)?;
    assert!(waits.is_none(), "{waits:?}");
    drop(release);
    let now = second.try_admit().map_err(io::Error::other)?;
    assert!(matches!(now, Some(Some(_))), "{now:?}");
    Ok(())
}
