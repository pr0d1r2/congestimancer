//! [`Scoped`]: a host [`Source`] with the cgroup's own files laid over the
//! signals they answer (src/cgroup:T4, V2).

use std::time::Instant;

use super::{CgroupSignal, Scope};
use crate::signal::{Measure, Reading, Root, Sampled, SignalId, Source};

/// The signals a cgroup answers from its own files: pressure, memory left
/// and `--cpu-throttle`.
pub const CGROUP_SIGNALS: [SignalId; 7] = [
    SignalId::CpuPressure,
    SignalId::MemPressure,
    SignalId::MemPressureFull,
    SignalId::IoPressure,
    SignalId::IoPressureFull,
    SignalId::Mem,
    SignalId::CpuThrottle,
];

/// A host [`Source`] read through a [`Scope`].
///
/// In a cgroup, every pressure gate reads the cgroup's `*.pressure`
/// files, and available memory is the cgroup's limit left, never more
/// than the host has (src/cgroup:V2); without a memory limit it is the
/// host's. `--cpu-throttle` reads the cgroup's `cpu.stat`, and on the host
/// is unsupported with the reason no cgroup is read. Every other signal
/// comes from the host.
#[derive(Debug)]
pub struct Scoped<H> {
    /// Where cgroup signals come from.
    scope: Scope,
    /// The cgroup's own signals, in a cgroup scope.
    cgroup: Option<Sampled<CgroupSignal>>,
    /// The host-wide source.
    host: H,
}

impl<H> Scoped<H> {
    /// Reads `host` through `scope`, the cgroup's files under `root`.
    #[must_use]
    pub fn new(root: Root, scope: Scope, host: H) -> Self {
        let cgroup = scope.cgroup().map(|cgroup| {
            Sampled::new(CgroupSignal::new(root, cgroup.clone()))
        });
        Self {
            scope,
            cgroup,
            host,
        }
    }

    /// Where cgroup signals come from.
    #[must_use]
    pub const fn scope(&self) -> &Scope {
        &self.scope
    }

    /// The host-wide source.
    pub const fn host_mut(&mut self) -> &mut H {
        &mut self.host
    }
}

impl<H: Source> Source for Scoped<H> {
    fn read(&mut self, id: SignalId, now: Instant) -> Reading {
        let owned = CGROUP_SIGNALS.contains(&id);
        match (&mut self.cgroup, &self.scope) {
            (Some(cgroup), _) if id == SignalId::Mem => {
                let own = cgroup.read(id, now);
                lesser(own, || self.host.read(id, now))
            }
            (Some(cgroup), _) if owned => cgroup.read(id, now),
            (None, Scope::Host { reason }) if id == SignalId::CpuThrottle => {
                Reading::unsupported(format!(
                    "--cpu-throttle needs a cgroup v2: {reason}"
                ))
            }
            _ => self.host.read(id, now),
        }
    }
}

/// The lesser of the cgroup's memory left and the host's available
/// memory; the host's alone when the cgroup has no limit, the cgroup's
/// alone when the host has no value.
fn lesser(own: Reading, host: impl FnOnce() -> Reading) -> Reading {
    let Some(&Measure::Bytes(own)) = own.measure() else {
        return host();
    };
    let host = match host().measure() {
        Some(&Measure::Bytes(host)) => Some(host),
        _ => None,
    };
    Reading::Value(Measure::Bytes(host.map_or(own, |host| own.min(host))))
}

#[cfg(test)]
mod tests;
