//! [`DiskSignal`]: `--disk` from `statvfs` on a path (src/disk:T1).

use std::path::{Path, PathBuf};

use crate::signal::{Measure, Reason, Sample, Signal, SignalId};
use crate::sys::{FsStats, statvfs};
use crate::units::Bytes;

/// The disk signals of the filesystem holding one path: `--disk`. Wrap
/// it in [`Sampled`](crate::signal::Sampled) for a
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

impl Signal for DiskSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let sample = match id {
            SignalId::Disk => statvfs(&self.path)
                .map(|stats| Measure::Bytes(free_space(&stats)))
                .map_err(|err| {
                    Reason::from(format!("{}: {err}", self.path.display()))
                }),
            _ => Err(Reason::from(format!("{id} is not a disk signal"))),
        };
        sample.map_or_else(Sample::Unsupported, Sample::Gauge)
    }
}

#[cfg(test)]
mod tests;
