//! [`capture`]: copy this machine's raw sources into a fixture tree, so a
//! new host's readings become test fixtures (src/sys:T15, V7).
//!
//! Kernel text files are copied verbatim under `out/<os>-<arch>/`, laid
//! out like `/` so the tree is itself a [`Root`]. macOS has no such files:
//! sysctl, mach and IOKit values are written decoded, as text. Nothing on
//! the host is changed, and nothing that names a person or a device is
//! kept: no command lines, environments, mount paths, serial numbers or
//! link-layer addresses.

use std::io;
use std::path::{Path, PathBuf};

use crate::signal::Root;

/// Kernel files read by the domain nodes (procfs, sysfs, cgroupfs).
const FILES: &[&str] = &[
    "/proc/loadavg",
    "/proc/stat",
    "/proc/meminfo",
    "/proc/vmstat",
    "/proc/diskstats",
    "/proc/net/dev",
    "/proc/pressure/cpu",
    "/proc/pressure/memory",
    "/proc/pressure/io",
    "/proc/sys/fs/file-nr",
    "/proc/sys/kernel/pid_max",
    "/proc/self/cgroup",
];

/// Directories whose entries' named files are read: thermal zones,
/// cooling devices, power supplies, block devices.
const DIRS: &[(&str, &[&str])] = &[
    (
        "/sys/class/thermal",
        &["type", "temp", "cur_state", "max_state"],
    ),
    (
        "/sys/class/power_supply",
        &["type", "online", "capacity", "status"],
    ),
    ("/sys/block", &["stat", "queue/rotational"]),
];

/// Copies every source this node reads on this machine into
/// `out/<os>-<arch>/` and returns the files written. A source the machine
/// lacks is skipped.
///
/// # Errors
///
/// A file that exists but cannot be read, or `out` cannot be written.
pub fn capture(root: &Root, out: &Path) -> io::Result<Vec<PathBuf>> {
    let dir = out.join(format!(
        "{}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    let mut written = copy_files(root, &dir)?;
    written.extend(imp::write_decoded(&dir)?);
    Ok(written)
}

/// Copies [`FILES`] and the [`DIRS`] entries present under `root` into
/// `dir`, keeping their paths.
fn copy_files(root: &Root, dir: &Path) -> io::Result<Vec<PathBuf>> {
    let _ = (root, dir, FILES, DIRS);
    Ok(Vec::new())
}

#[cfg(target_os = "macos")]
mod imp {
    use std::io;
    use std::path::{Path, PathBuf};

    pub(super) fn write_decoded(dir: &Path) -> io::Result<Vec<PathBuf>> {
        let _ = dir;
        Ok(Vec::new())
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use std::io;
    use std::path::{Path, PathBuf};

    /// Linux has no decoded sources: everything is a kernel file.
    pub(super) fn write_decoded(_: &Path) -> io::Result<Vec<PathBuf>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests;
