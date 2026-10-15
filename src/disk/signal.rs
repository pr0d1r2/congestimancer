//! [`DiskSignal`]: `--disk` and `--inodes` from `statvfs` on a path
//! (src/disk:T1, T2).

use std::path::{Path, PathBuf};

use crate::signal::{Measure, Reason, Sample, Signal, SignalId};
use crate::sys::{FsStats, statvfs};
use crate::units::{Bytes, Count};

/// The disk signals of the filesystem holding one path: `--disk` and
/// `--inodes`. Wrap it in [`Sampled`](crate::signal::Sampled) for a
/// [`Source`](crate::signal::Source).
///
/// Both OSes read `statvfs` on the path itself, never the mount table
/// (src/disk:V4). Any other signal is unsupported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskSignal {
    /// The path whose filesystem is read.
    path: PathBuf,
}

impl DiskSignal {
    /// Reads the filesystem holding `path`.
    #[must_use]
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    /// The path whose filesystem is read.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// `--disk`: the bytes an unprivileged user may still write,
/// `f_bavail * f_frsize` (src/disk:V1).
#[must_use]
pub const fn free_space(stats: &FsStats) -> Bytes {
    Bytes(stats.bytes_available())
}

/// `--inodes`: the inodes an unprivileged user may still take, `f_favail`
/// (src/disk:V2); `None` on a filesystem that does not count inodes.
#[must_use]
pub const fn free_inodes(stats: &FsStats) -> Option<Count> {
    if stats.files == 0 {
        None
    } else {
        Some(Count(stats.files_available))
    }
}

impl DiskSignal {
    /// The stats of the filesystem holding the path.
    fn stats(&self) -> Result<FsStats, Reason> {
        statvfs(&self.path).map_err(|err| {
            Reason::from(format!("{}: {err}", self.path.display()))
        })
    }

    /// `--inodes`, or why there is none.
    fn inodes(&self) -> Result<Measure, Reason> {
        free_inodes(&self.stats()?)
            .map(Measure::Count)
            .ok_or_else(|| {
                Reason::from(format!(
                    "{}: filesystem does not count inodes",
                    self.path.display()
                ))
            })
    }
}

impl Signal for DiskSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let sample = match id {
            SignalId::Disk => {
                self.stats().map(|stats| Measure::Bytes(free_space(&stats)))
            }
            SignalId::Inodes => self.inodes(),
            _ => Err(Reason::from(format!("{id} is not a disk signal"))),
        };
        sample.map_or_else(Sample::Unsupported, Sample::Gauge)
    }
}

#[cfg(test)]
mod tests;
