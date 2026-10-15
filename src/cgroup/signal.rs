//! [`CgroupSignal`]: the signals a cgroup v2 answers from its own files
//! (src/cgroup:T3, V4).

use super::{Cgroup, parse_cpu_stat};
use crate::signal::{Counter, Reason, Root, Sample, Signal, SignalId};
use crate::sys::read_text;

/// The signals one cgroup answers from its own files, read under a
/// [`Root`]; wrap it in [`Sampled`](crate::signal::Sampled) for a
/// [`Source`](crate::signal::Source).
///
/// `--cpu-throttle` reads `cpu.stat` `throttled_usec` as busy time, so it
/// reads as the share of the last interval the quota stopped the cgroup
/// (src/cgroup:V4). Any signal it does not own is unsupported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CgroupSignal {
    /// The directory that stands for `/`.
    root: Root,
    /// The cgroup read.
    cgroup: Cgroup,
}

impl CgroupSignal {
    /// Reads `cgroup` under `root`.
    #[must_use]
    pub const fn new(root: Root, cgroup: Cgroup) -> Self {
        Self { root, cgroup }
    }

    /// The cgroup read.
    #[must_use]
    pub const fn cgroup(&self) -> &Cgroup {
        &self.cgroup
    }
}

impl CgroupSignal {
    /// The text of the cgroup's file `name`, or why it is unreadable.
    fn read(&self, name: &str) -> Result<String, Reason> {
        read_text(&self.root, &self.cgroup.file(name))
            .map_err(|err| Reason::from(err.to_string()))
    }

    /// `--cpu-throttle`: time the quota stopped the cgroup, as busy time.
    fn throttle(&self) -> Result<Sample, Reason> {
        let stat = parse_cpu_stat(&self.read("cpu.stat")?)
            .map_err(|err| Reason::from(err.to_string()))?;
        let throttling = stat.throttling.ok_or_else(|| {
            let path = self.cgroup.path();
            Reason::from(format!("{path} has no cpu controller to throttle it"))
        })?;
        let busy = throttling.throttled;
        Ok(Sample::Counter(Counter::Busy { busy }))
    }
}

impl Signal for CgroupSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let sample = match id {
            SignalId::CpuThrottle => self.throttle(),
            _ => Err(Reason::from(format!("{id} is not a cgroup signal"))),
        };
        sample.unwrap_or_else(Sample::Unsupported)
    }
}

#[cfg(test)]
mod tests;
