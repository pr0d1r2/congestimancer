//! [`LimitsSignal`]: `--fds` and `--pids`, from Linux `/proc` under a
//! [`Root`] or macOS sysctls (src/limits:T1, T2, T3).

use std::io;

use super::{limit_share, parse_file_nr};
use crate::signal::{Measure, Reason, Root, Sample, Signal, SignalId};
use crate::sys::read_text;
use crate::units::Percent;

/// Linux file handles: allocated, unused, maximum.
const FILE_NR: &str = "/proc/sys/fs/file-nr";

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
            Os::Macos => sysctl_share(Ok(0), Ok(0), "kern.maxfiles"),
        }
    }
}

impl Signal for LimitsSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let sample = match id {
            SignalId::Fds => self.fds(),
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
    let _ = (used, max, limit);
    Err(Reason::from("stub"))
}

/// An error as the reason a signal has no value.
fn reason(err: impl ToString) -> Reason {
    Reason::from(err.to_string())
}

#[cfg(test)]
mod tests;
