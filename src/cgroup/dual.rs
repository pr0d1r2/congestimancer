//! [`Dual`]: `status` shows a cgroup's values beside the host's
//! (src/cgroup:T7, V6).

use std::fmt;
use std::time::Instant;

use super::{CGROUP_SIGNALS, Scope, Scoped};
use crate::signal::{Reading, SignalId, SignalSet, Source};

/// One signal as the cgroup and the host read it, for `status`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Dual {
    /// The signal.
    pub id: SignalId,
    /// What gates read in this scope; unsupported with the reason when no
    /// cgroup is read.
    pub cgroup: Reading,
    /// The host-wide value.
    pub host: Reading,
}

impl Dual {
    /// `id` read as `cgroup` and as `host`.
    #[must_use]
    pub const fn new(id: SignalId, cgroup: Reading, host: Reading) -> Self {
        Self { id, cgroup, host }
    }
}

/// Renders `cpu-pressure: cgroup 12.5% | host 50%`.
impl fmt::Display for Dual {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { id, cgroup, host } = self;
        write!(f, "{id}: cgroup {cgroup} | host {host}")
    }
}

impl<H: Source> Scoped<H> {
    /// Every signal of `ids` a cgroup has its own files for, read in this
    /// scope beside the host's value, in [`CGROUP_SIGNALS`] order.
    pub fn dual(&mut self, ids: SignalSet, now: Instant) -> Vec<Dual> {
        CGROUP_SIGNALS
            .into_iter()
            .filter(|&id| ids.contains(id))
            .map(|id| {
                let cgroup = match self.scope() {
                    Scope::Host { reason } => {
                        Reading::unsupported(format!("no cgroup: {reason}"))
                    }
                    Scope::Cgroup(_) => self.read(id, now),
                };
                Dual::new(id, cgroup, self.host_mut().read(id, now))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
