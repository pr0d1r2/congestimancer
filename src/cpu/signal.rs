//! [`CpuSignal`]: the CPU signals, from Linux `/proc` under a [`Root`]
//! or macOS `sysinfo` and mach calls (src/cpu:T2-T8).

use super::Loads;
use crate::signal::{Reason, Root, Sample, Signal, SignalId};
use crate::sys::LoadAverage;

/// Which OS's sources a [`CpuSignal`] reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
    /// `/proc` under the root.
    Linux,
    /// `sysinfo` and mach calls through `crate::sys`.
    Macos,
}

/// The CPU signals. Wrap it in [`Sampled`](crate::signal::Sampled) for a
/// [`Source`](crate::signal::Source), and that in
/// [`Scoped`](crate::cgroup::Scoped) so a cgroup answers
/// `--cpu-pressure` from its own `cpu.pressure` (src/cpu:V5).
///
/// [`SignalId::Load`] is the whole machine's load average, the reading
/// behind `--load` and `--all-load`; `--load` divides it by
/// [`cpu_count`](super::cpu_count) with [`load_per_cpu`](super::load_per_cpu)
/// (src/cpu:V1). Any signal of another domain is unsupported.
#[derive(Debug)]
pub struct CpuSignal {
    /// The directory that stands for `/`.
    root: Root,
    /// The OS whose sources are read.
    os: Os,
}

impl CpuSignal {
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

impl Signal for CpuSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let _ = (&self.root, self.os, loads_from);
        Sample::Unsupported(Reason::from(format!("{id} is not a cpu signal")))
    }
}

/// The load averages `sysinfo` gives, to thousandths; a negative or
/// non-finite average is unsupported.
fn loads_from(load: LoadAverage) -> Result<Loads, Reason> {
    let _ = load;
    Err(Reason::from("todo"))
}

#[cfg(test)]
mod tests;
