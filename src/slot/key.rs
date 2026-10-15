//! The slot key: `--slot-key K`, else the command's basename made safe
//! (src/slot:V3, V10).

use super::SlotError;
use crate::units::{Name, parse_name};

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
    let (input, candidate) = match (explicit, cmd) {
        (Some(key), _) => (key, key.to_owned()),
        (None, Some(cmd)) => (cmd, sanitise(basename(cmd))),
        (None, None) => ("", String::new()),
    };
    parse_name(&candidate).map_err(|_| SlotError::BadKey {
        input: input.to_owned(),
    })
}

/// The last `/`-separated part of `cmd`.
fn basename(cmd: &str) -> &str {
    cmd.rsplit('/').next().unwrap_or(cmd)
}

/// `s` with every character outside `A-Z a-z 0-9 . _ -` and a leading `-`
/// turned to `_`, cut to 64 characters.
fn sanitise(s: &str) -> String {
    let safe = |(i, c): (usize, char)| {
        let allowed = c.is_ascii_alphanumeric() || "._-".contains(c);
        if allowed && !(i == 0 && c == '-') {
            c
        } else {
            '_'
        }
    };
    s.chars().enumerate().map(safe).take(MAX_LEN).collect()
}

/// The longest a key may be (src/units:V13).
const MAX_LEN: usize = 64;

#[cfg(test)]
mod tests;
