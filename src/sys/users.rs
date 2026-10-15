//! Users: names and ids from the password database (`getpwnam_r`,
//! `getpwuid_r`) and this process's uid, for counts by user
//! (src/sys:T18, src/procs).

use std::io;

/// The uid of the user called `name`; `None` when there is no such user.
///
/// # Errors
///
/// [`io::ErrorKind::InvalidInput`] for a name with a NUL; a lookup
/// failure other than "not found" as the OS error.
pub fn user_id(name: &str) -> io::Result<Option<u32>> {
    let _ = name;
    Ok(None)
}

/// The name of the user with `uid`; `None` when no user has it.
///
/// # Errors
///
/// [`io::ErrorKind::InvalidData`] for a name not UTF-8; a lookup failure
/// other than "not found" as the OS error.
pub fn user_name(uid: u32) -> io::Result<Option<String>> {
    let _ = uid;
    Ok(None)
}

/// The real uid of this process.
#[must_use]
pub fn current_uid() -> u32 {
    u32::MAX
}

#[cfg(test)]
mod tests;
