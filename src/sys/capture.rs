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

use super::files::read_text;
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
    let mut sources: Vec<String> =
        FILES.iter().map(|f| (*f).to_owned()).collect();
    for (parent, names) in DIRS {
        sources.extend(entries(root, parent, names));
    }
    let out = Root::new(dir);
    let mut written = Vec::new();
    for source in sources {
        // Absent, unreadable or not text: this machine lacks the source.
        let Ok(text) = read_text(root, &source) else {
            continue;
        };
        written.push(write(&out.join(&source), &text)?);
    }
    Ok(written)
}

/// `parent/<entry>/<name>` for each entry of `parent` under `root`, in
/// name order; nothing when `parent` is absent.
fn entries(root: &Root, parent: &str, names: &[&str]) -> Vec<String> {
    listing(&root.join(parent))
        .iter()
        .flat_map(|entry| {
            names
                .iter()
                .map(move |name| format!("{parent}/{entry}/{name}"))
        })
        .collect()
}

/// The entry names of directory `dir`, sorted; none if it cannot be read.
fn listing(dir: &Path) -> Vec<String> {
    let Ok(listing) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<String> = listing
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    found.sort();
    found
}

/// Writes `text` to `path`, making its directories.
fn write(path: &Path, text: &str) -> io::Result<PathBuf> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(path.to_path_buf())
}

#[cfg(target_os = "macos")]
mod imp {
    use std::fmt::Debug;
    use std::io;
    use std::path::{Path, PathBuf};

    use super::write;
    use crate::sys::props::render_props;
    use crate::sys::{disk, gpu, mach, net, notify, power, sysctl};

    /// Integer sysctls the domain nodes read.
    const SYSCTLS: &[&str] = &[
        "kern.memorystatus_vm_pressure_level",
        "kern.memorystatus_level",
        "kern.num_files",
        "kern.maxfiles",
        "kern.num_tasks",
        "kern.maxproc",
        "hw.pagesize",
    ];

    pub(super) fn write_decoded(dir: &Path) -> io::Result<Vec<PathBuf>> {
        let texts = [
            ("sysctl", sysctls()),
            ("mach", mach_text()),
            ("net", show(net::interface_counters())),
            ("iokit/block", show(disk::block_devices())),
            ("iokit/gpu", gpu_text()),
            ("iokit/power", power_text()),
        ];
        texts
            .iter()
            .map(|(name, text)| write(&dir.join(name), text))
            .collect()
    }

    /// A reading as pretty `Debug` text, or its error.
    fn show<T: Debug>(reading: io::Result<T>) -> String {
        reading.map_or_else(
            |err| format!("error: {err}\n"),
            |v| format!("{v:#?}\n"),
        )
    }

    fn sysctls() -> String {
        let mut text: String = SYSCTLS
            .iter()
            .map(|name| format!("{name} = {}", show(sysctl::sysctl_int(name))))
            .collect();
        text.push_str(&show(sysctl::swap_usage()));
        text
    }

    fn mach_text() -> String {
        let pset = show(mach::pset_load());
        let vm = show(mach::vm_stats());
        format!("{}{vm}{pset}", show(mach::cpu_ticks()))
    }

    fn gpu_text() -> String {
        let gpus = gpu::gpu_usage().unwrap_or_default();
        gpus.iter().map(|g| render_props(&g.stats) + "\n").collect()
    }

    /// Power sources without their serial numbers, plus notify state.
    fn power_text() -> String {
        let (providing, sources) = power::sources().unwrap_or_default();
        let mut text = format!("providing = {providing:?}\n");
        for mut source in sources {
            source.remove("Hardware Serial Number");
            text.push_str(&render_props(&source));
        }
        for name in [notify::THERMAL_PRESSURE, notify::LOW_POWER_MODE] {
            text.push_str(&format!(
                "{name} = {}",
                show(notify::notify_state(name))
            ));
        }
        text + &show(power::hid_idle_ns())
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
