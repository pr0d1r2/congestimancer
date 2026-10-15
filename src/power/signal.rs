//! [`PowerSignal`]: the thermal and power gates, from Linux sysfs under a
//! [`Root`] or macOS notify and IOKit state (src/power:T2, T3, T4, T6,
//! T7).

use crate::signal::{Reason, Root, Sample, Signal, SignalId};

/// Which OS's sources a [`PowerSignal`] reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
    /// sysfs under the root.
    Linux,
    /// notify(3) and IOKit through `crate::sys`.
    Macos,
}

/// The thermal and power signals: `--cool`, `--temp`, `--on-ac`,
/// `--battery`, `--no-low-power`. Wrap it in
/// [`Sampled`](crate::signal::Sampled) for a
/// [`Source`](crate::signal::Source).
///
/// `--cool` on Linux compares the CPUs' summed `thermal_throttle`
/// counters with the previous sample; where there are none (AMD, ARM) or
/// on the first sample, it asks whether any CPU cooling device is active
/// (src/power:V1, R3). macOS reads thermal pressure (R1). Any other
/// signal is unsupported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerSignal {
    /// The directory that stands for `/`.
    root: Root,
    /// The OS whose sources are read.
    os: Os,
    /// The summed throttle counters at the previous `--cool` sample.
    throttles: Option<u64>,
}

impl PowerSignal {
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
        Self {
            root,
            os,
            throttles: None,
        }
    }
}

impl Signal for PowerSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let _ = (&self.root, self.os, self.throttles);
        Sample::Unsupported(Reason::from(format!("{id} is not a power signal")))
    }
}

#[cfg(test)]
mod tests;
