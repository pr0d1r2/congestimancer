//! [`read_text`]: kernel text files (procfs, sysfs, cgroupfs) read under a
//! [`Root`], so tests and `CGMR_FIXTURE_ROOT` read a fixture tree
//! (src/sys:T2, src/signal:V5).

use std::io;

use crate::signal::Root;

/// The text of the absolute `path` (such as `/proc/loadavg`) under `root`.
///
/// # Errors
///
/// A file that does not exist is [`io::ErrorKind::Unsupported`] naming the
/// path: this machine has no such source (src/sys:V4). Any other failure
/// (permission, not UTF-8) passes through with its own kind.
pub fn read_text(root: &Root, path: &str) -> io::Result<String> {
    let _ = (root, path);
    Err(crate::sys::error::unsupported("RED"))
}

#[cfg(test)]
mod tests;
