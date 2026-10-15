//! [`read_text`] and [`list_dir`]: kernel text files and directories (procfs, sysfs, cgroupfs) read under a
//! [`Root`], so tests and `CGMR_FIXTURE_ROOT` read a fixture tree
//! (src/sys:T2, src/signal:V5).

use std::ffi::OsString;
use std::io;

use super::error::unsupported;
use crate::signal::Root;

/// The text of the absolute `path` (such as `/proc/loadavg`) under `root`.
///
/// # Errors
///
/// A file that does not exist is [`io::ErrorKind::Unsupported`] naming the
/// path: this machine has no such source (src/sys:V4). Any other failure
/// (permission, not UTF-8) passes through with its own kind.
pub fn read_text(root: &Root, path: &str) -> io::Result<String> {
    std::fs::read_to_string(root.join(path)).map_err(|err| absent(path, err))
}

/// The entry names of the absolute directory `path` (such as
/// `/sys/class/power_supply`) under `root`, sorted, without `.` and `..`.
///
/// # Errors
///
/// A directory that does not exist is [`io::ErrorKind::Unsupported`]
/// naming the path (src/sys:V4). Any other failure (permission, not a
/// directory, a name not UTF-8) passes through with its own kind.
pub fn list_dir(root: &Root, path: &str) -> io::Result<Vec<String>> {
    let entries =
        std::fs::read_dir(root.join(path)).map_err(|err| absent(path, err))?;
    let mut names = entries
        .map(|entry| utf8_name(path, entry?.file_name()))
        .collect::<io::Result<Vec<_>>>()?;
    names.sort_unstable();
    Ok(names)
}

/// `err`, or [`io::ErrorKind::Unsupported`] naming `path` when it is
/// [`io::ErrorKind::NotFound`] (src/sys:V4).
fn absent(path: &str, err: io::Error) -> io::Error {
    if err.kind() == io::ErrorKind::NotFound {
        unsupported(format_args!("{path}: not present"))
    } else {
        err
    }
}

/// A directory entry name of `path` as UTF-8, or
/// [`io::ErrorKind::InvalidData`] naming it.
fn utf8_name(path: &str, name: OsString) -> io::Result<String> {
    name.into_string().map_err(|name| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{path}: entry {} is not UTF-8", name.display()),
        )
    })
}

#[cfg(test)]
mod tests;
