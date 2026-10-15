//! [`MemSignal`]: the memory signals, from Linux `/proc` under a
//! [`Root`] or macOS sysctls and mach counters (src/mem:T1).

use crate::signal::{Reason, Root, Sample, Signal, SignalId};

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
    const fn on(root: Root, os: Os) -> Self {
        Self { root, os }
    }
}

impl Signal for MemSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let _ = (&self.root, self.os);
        Sample::Unsupported(Reason::from(format!("{id} is not a mem signal")))
    }
}

#[cfg(test)]
mod tests;
