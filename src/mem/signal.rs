//! [`MemSignal`]: the memory signals, from Linux `/proc` under a
//! [`Root`] or macOS sysctls and mach counters (src/mem:T1).

use crate::signal::{Measure, Reason, Root, Sample, Signal, SignalId};
use crate::sys::Snapshot;
use crate::units::Bytes;

/// Which OS's sources a [`MemSignal`] reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
    /// `/proc` under the root.
    Linux,
    /// sysctls and mach counters through `crate::sys`.
    Macos,
}

/// The memory signals. Wrap it in [`Sampled`](crate::signal::Sampled)
/// for a [`Source`](crate::signal::Source).
///
/// [`SignalId::Mem`] is available memory, never "free" (src/mem:V1),
/// the one reading behind `--mem`, `--all-mem` and `--needs-mem`. Any
/// signal of another domain is unsupported.
#[derive(Debug)]
pub struct MemSignal {
    /// The directory that stands for `/`.
    root: Root,
    /// The OS whose sources are read.
    os: Os,
    /// The `sysinfo` view available memory is read from.
    snapshot: Snapshot,
}

impl MemSignal {
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
    fn on(root: Root, os: Os) -> Self {
        Self {
            root,
            os,
            snapshot: Snapshot::new(),
        }
    }
}

impl MemSignal {
    /// Available memory: what new work can take without swapping, never
    /// "free", from `sysinfo` on both OSes (Linux `MemAvailable`, the
    /// macOS native estimate; src/mem:V1).
    fn available(&mut self) -> Bytes {
        self.snapshot.refresh_memory();
        Bytes(self.snapshot.memory().available)
    }
}

impl Signal for MemSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let _ = (&self.root, self.os);
        match id {
            SignalId::Mem => Sample::Gauge(Measure::Bytes(self.available())),
            _ => Sample::Unsupported(Reason::from(format!(
                "{id} is not a mem signal"
            ))),
        }
    }
}

#[cfg(test)]
mod tests;
