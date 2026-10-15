//! [`LimitsSignal`]: `--fds` and `--pids`, from Linux `/proc` under a
//! [`Root`] or macOS sysctls (src/limits:T1, T2, T3).

use crate::signal::{Reason, Root, Sample, Signal, SignalId};

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

impl Signal for LimitsSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let _ = (id, &self.root, self.os);
        Sample::Unsupported(Reason::from("stub"))
    }
}

#[cfg(test)]
mod tests;
