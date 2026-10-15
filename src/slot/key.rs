//! The slot key: `--slot-key K`, else the command's basename made safe
//! (src/slot:V3, V10).

use super::SlotError;
use crate::units::Name;

/// The slot key: `explicit` (`--slot-key`) when given, else the basename
/// of `cmd`, its unsafe characters turned to `_` and cut to 64, so a key
/// never escapes the lock dir (src/slot:V3, V10).
///
/// # Errors
///
/// [`SlotError::BadKey`] when `explicit` is not a [`Name`], when there is
/// no command, or when its basename makes no name (`.`, `..`, empty).
pub fn slot_key(
    explicit: Option<&str>,
    cmd: Option<&str>,
) -> Result<Name, SlotError> {
    let _ = (explicit, cmd);
    Err(SlotError::BadKey {
        input: String::new(),
    })
}

#[cfg(test)]
mod tests;
