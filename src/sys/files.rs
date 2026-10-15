//! [`read_text`] and [`list_dir`]: kernel text files and directories (procfs, sysfs, cgroupfs) read under a
//! [`Root`], so tests and `CGMR_FIXTURE_ROOT` read a fixture tree
//! (src/sys:T2, src/signal:V5).

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
    std::fs::read_to_string(root.join(path)).map_err(|err| {
        if err.kind() == io::ErrorKind::NotFound {
            unsupported(format_args!("{path}: not present"))
        } else {
            err
        }
    })
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
    let _ = (root, path);
    Err(io::Error::other("stub"))
}

#[cfg(test)]
mod tests;
