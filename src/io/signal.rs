//! [`IoSignal`]: the IO signals of one block device, from Linux
//! `/proc/diskstats` under a [`Root`] or the macOS IOKit block storage
//! counters (src/io:T4-T9).

use std::path::{Path, PathBuf};
use std::time::Duration;

use super::diskstats::DISKSTATS;
use super::mount::MOUNTINFO;
use super::{
    DiskStat, IoError, linux_device, macos_device, parse_diskstats,
    parse_mountinfo,
};
use crate::signal::{Counter, Reason, Root, Sample, Signal, SignalId};
use crate::sys::{BlockStats, mount_source, physical_disk_stats, read_text};

/// Which OS's sources an [`IoSignal`] reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
    /// `/proc` under the root.
    Linux,
    /// IOKit through `crate::sys`.
    Macos,
}

/// The IO signals of the block device holding a path, or of the device
/// `--io-device` names (src/io:V1). Wrap it in
/// [`Sampled`](crate::signal::Sampled) for a
/// [`Source`](crate::signal::Source): rates, busy and latency are
/// counters, so the first reading is warming up (src/io:V4); wrap that
/// in [`Scoped`](crate::cgroup::Scoped) so a cgroup answers
/// `--io-pressure` from its own `io.pressure` (src/io:V9).
///
/// A path on tmpfs, NFS or FUSE has no block device: its signals are
/// unsupported, with a hint to name one with `--io-device` (src/io:V3).
/// Any signal of another domain is unsupported.
#[derive(Debug)]
pub struct IoSignal {
    /// The directory that stands for `/`.
    root: Root,
    /// The OS whose sources are read.
    os: Os,
    /// The path whose device is measured (`--disk-path`, default `.`).
    path: PathBuf,
}

impl IoSignal {
    /// Reads this OS's sources for the device holding `disk_path`, or the
    /// working directory without one; Linux files under `root`.
    #[must_use]
    pub fn new(root: Root, disk_path: Option<&Path>) -> Self {
        let os = if cfg!(target_os = "macos") {
            Os::Macos
        } else {
            Os::Linux
        };
        Self::on(root, os, disk_path)
    }

    /// Reads `os`'s sources for the device holding `disk_path`.
    fn on(root: Root, os: Os, disk_path: Option<&Path>) -> Self {
        Self {
            root,
            os,
            path: disk_path.unwrap_or_else(|| Path::new(".")).to_path_buf(),
        }
    }
}

impl Signal for IoSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        Sample::Unsupported(Reason::from(format!("{id} is not an io signal")))
    }
}

impl IoSignal {
    /// The device's running totals on this OS.
    fn totals(&self) -> Result<Totals, Reason> {
        match self.os {
            Os::Linux => linux_totals(&self.linux_disk()?),
            Os::Macos => self.macos_totals(),
        }
    }
}

impl IoSignal {
    /// The `/proc/diskstats` line of the measured device.
    fn linux_disk(&self) -> Result<DiskStat, Reason> {
        let text = read_text(&self.root, DISKSTATS).map_err(reason)?;
        let disks = parse_diskstats(&text).map_err(reason)?;
        let name = self.linux_device(&disks)?;
        let found = disks.into_iter().find(|d| d.name == name);
        found.ok_or_else(|| reason(IoError::UnknownDevice { name }))
    }

    /// The `/proc/diskstats` name of the device holding the path.
    fn linux_device(&self, disks: &[DiskStat]) -> Result<String, Reason> {
        let text = read_text(&self.root, MOUNTINFO).map_err(reason)?;
        let mounts = parse_mountinfo(&text).map_err(reason)?;
        let path = std::path::absolute(&self.path).map_err(reason)?;
        linux_device(&mounts, disks, &path).map_err(reason)
    }

    /// The IOKit counters of the physical disk measured (src/io:V2).
    fn macos_totals(&self) -> Result<Totals, Reason> {
        let source = mount_source(&self.path).map_err(reason)?;
        let name = macos_device(&source, &self.path).map_err(reason)?;
        block_totals(&physical_disk_stats(&name).map_err(reason)?)
    }
}

/// One device's running totals, whichever OS counted them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Totals {
    /// Bytes read.
    read: u64,
    /// Bytes written.
    written: u64,
    /// Time the device was busy.
    busy: Duration,
    /// Time spent on completed requests.
    time: Duration,
    /// Requests completed.
    ops: u64,
}

/// A Linux device's totals: busy is `io_ticks`, time is reading plus
/// writing time (src/io:V6, V7).
fn linux_totals(disk: &DiskStat) -> Result<Totals, Reason> {
    let past =
        || Reason::from(format!("{DISKSTATS} {} past 64 bits", disk.name));
    let ms = disk.read.ms.checked_add(disk.write.ms).ok_or_else(past)?;
    Ok(Totals {
        read: disk.read.bytes().ok_or_else(past)?,
        written: disk.write.bytes().ok_or_else(past)?,
        busy: Duration::from_millis(disk.io_ticks_ms),
        time: Duration::from_millis(ms),
        ops: disk.read.ops.checked_add(disk.write.ops).ok_or_else(past)?,
    })
}

/// A macOS disk's totals: busy is the read plus write total time, an
/// upper bound the sampler caps at 100% (src/io:V6, R1).
fn block_totals(stats: &BlockStats) -> Result<Totals, Reason> {
    let past = || Reason::from("IOKit block storage counters past 64 bits");
    let (r, w) = (stats.read, stats.write);
    let ns = r
        .total_time_ns
        .checked_add(w.total_time_ns)
        .ok_or_else(past)?;
    Ok(Totals {
        read: r.bytes,
        written: w.bytes,
        busy: Duration::from_nanos(ns),
        time: Duration::from_nanos(ns),
        ops: r.operations.checked_add(w.operations).ok_or_else(past)?,
    })
}

/// The counter `id` reads from `totals`.
fn counter(id: SignalId, totals: &Totals) -> Result<Counter, Reason> {
    let rate = |total| Counter::Rate { total };
    match id {
        SignalId::IoRead => Ok(rate(totals.read)),
        SignalId::IoWrite => Ok(rate(totals.written)),
        SignalId::Io => both(totals).map(rate),
        SignalId::IoBusy => Ok(Counter::Busy { busy: totals.busy }),
        SignalId::IoLatency => Ok(Counter::Mean {
            time: totals.time,
            events: totals.ops,
        }),
        _ => Err(Reason::from(format!("{id} is not an io counter"))),
    }
}

/// Bytes read plus written.
fn both(totals: &Totals) -> Result<u64, Reason> {
    let sum = totals.read.checked_add(totals.written);
    sum.ok_or_else(|| Reason::from("IO bytes past 64 bits"))
}

/// An error as the reason a signal has no value.
fn reason(err: impl ToString) -> Reason {
    Reason::from(err.to_string())
}

#[cfg(test)]
mod tests;
