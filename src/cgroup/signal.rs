//! [`CgroupSignal`]: the signals a cgroup v2 answers from its own files
//! (src/cgroup:T3, V4).

use crate::signal::{Root, Sample, Signal, SignalId};

use super::Cgroup;

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

impl Signal for CgroupSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let _ = (id, &self.root);
        Sample::Unsupported("RED".into())
    }
}

#[cfg(test)]
mod tests;
