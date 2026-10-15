//! [`LimitsSignal`]: `--fds` and `--pids`, from Linux `/proc` under a
//! [`Root`] or macOS sysctls (src/limits:T1, T2, T3).

use std::io;

use super::{limit_share, parse_file_nr, parse_loadavg_tasks, parse_pid_max};
use crate::signal::{Measure, Reason, Root, Sample, Signal, SignalId};
use crate::sys::{Snapshot, read_text, sysctl_int};
use crate::units::Percent;

/// Linux file handles: allocated, unused, maximum.
const FILE_NR: &str = "/proc/sys/fs/file-nr";

/// Linux's highest pid plus one.
const PID_MAX: &str = "/proc/sys/kernel/pid_max";

/// Linux load averages, with the task count after the `/`.
const LOADAVG: &str = "/proc/loadavg";

/// Which OS's sources a [`LimitsSignal`] reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
    /// `/proc` under the root.
    Linux,
    /// sysctls through `crate::sys`.
    Macos,
}

/// The system limits signals: `--fds` and `--pids`, each the share of its
/// system maximum in use. Wrap it in
/// [`Sampled`](crate::signal::Sampled) for a
/// [`Source`](crate::signal::Source).
///
/// A maximum of 0, or one that cannot be read, is unsupported, never a
/// division (src/limits:V3). Any other signal is unsupported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitsSignal {
    /// The directory that stands for `/`.
    root: Root,
    /// The OS whose sources are read.
    os: Os,
}

impl LimitsSignal {
    /// Reads this OS's sources, Linux files under `root`.
    #[must_use]
    pub fn new(root: Root) -> Self {
        let os = if cfg!(target_os = "macos") {
            Os::Macos
        } else {
            Os::Linux
        };
        Self::on(root, os)
    }

    /// Reads `os`'s sources.
    const fn on(root: Root, os: Os) -> Self {
        Self { root, os }
    }
}

impl LimitsSignal {
    /// `--fds`: open files as a share of the system maximum
    /// (src/limits:V1).
    fn fds(&self) -> Result<Percent, Reason> {
        match self.os {
            Os::Linux => {
                let text = read_text(&self.root, FILE_NR).map_err(reason)?;
                let nr = parse_file_nr(&text).map_err(reason)?;
                share(nr.open(), nr.max, "fs.file-max")
            }
            Os::Macos => sysctl_share(
                sysctl_int("kern.num_files"),
                sysctl_int("kern.maxfiles"),
                "kern.maxfiles",
            ),
        }
    }

    /// `--pids`: processes as a share of the system maximum
    /// (src/limits:V2). Linux counts tasks, since every thread holds a
    /// pid below `pid_max`; macOS counts processes against
    /// `kern.maxproc` (`kern.num_tasks` is a fixed cap, not a count).
    fn pids(&self) -> Result<Percent, Reason> {
        match self.os {
            Os::Linux => {
                let text = read_text(&self.root, PID_MAX).map_err(reason)?;
                let max = parse_pid_max(&text).map_err(reason)?;
                let text = read_text(&self.root, LOADAVG).map_err(reason)?;
                let tasks = parse_loadavg_tasks(&text).map_err(reason)?;
                share(tasks, max, "kernel.pid_max")
            }
            Os::Macos => sysctl_share(
                process_count(),
                sysctl_int("kern.maxproc"),
                "kern.maxproc",
            ),
        }
    }
}

/// The processes running now, through the `crate::sys` snapshot.
fn process_count() -> io::Result<i64> {
    let mut snapshot = Snapshot::new();
    snapshot.refresh_processes();
    i64::try_from(snapshot.processes().len())
        .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
}

impl Signal for LimitsSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let sample = match id {
            SignalId::Fds => self.fds(),
            SignalId::Pids => self.pids(),
            _ => Err(Reason::from(format!("{id} is not a limits signal"))),
        };
        sample.map_or_else(Sample::Unsupported, |p| {
            Sample::Gauge(Measure::Percent(p))
        })
    }
}

/// `used` as a share of `max`; a `max` of 0, named by `limit`, is
/// unsupported (src/limits:V3).
fn share(used: u64, max: u64, limit: &str) -> Result<Percent, Reason> {
    limit_share(used, max).ok_or_else(|| Reason::from(format!("{limit} is 0")))
}

/// The sysctl counts `used` of `max` as a share; a failed read, a
/// negative count or a `max` of 0, named by `limit`, is unsupported
/// (src/limits:V3).
fn sysctl_share(
    used: io::Result<i64>,
    max: io::Result<i64>,
    limit: &str,
) -> Result<Percent, Reason> {
    let count = |n: i64| {
        u64::try_from(n)
            .map_err(|_| Reason::from(format!("negative sysctl count {n}")))
    };
    let used = count(used.map_err(reason)?)?;
    let max = count(max.map_err(reason)?)?;
    share(used, max, limit)
}

/// An error as the reason a signal has no value.
fn reason(err: impl ToString) -> Reason {
    Reason::from(err.to_string())
}

#[cfg(test)]
mod tests;
