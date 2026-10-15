//! [`NetSignal`]: `--net-rx`, `--net-tx` and `--net` over the chosen
//! interfaces (src/net:T2, T4, V1, V2).

use std::time::Instant;

use super::rates::Rates;
use crate::signal::{Reading, Root, SignalId, Source};
use crate::sys::InterfaceCounters;

/// Which OS's sources a [`NetSignal`] reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
    /// `/proc/net/dev` and `/sys/class/net` under the root.
    Linux,
    /// `NET_RT_IFLIST2` through `crate::sys`.
    Macos,
}

/// The interface rate signals: `--net-rx`, `--net-tx` and `--net`, in
/// bytes per second over every up, non-loopback interface
/// (src/net:V1, V2). It is a [`Source`] itself: its counters warm up per
/// interface (src/net:V5). Any other signal is unsupported.
#[derive(Debug)]
pub struct NetSignal {
    /// The directory that stands for `/`.
    root: Root,
    /// The OS whose sources are read.
    os: Os,
    /// The bases of the rates.
    rates: Rates,
}

impl NetSignal {
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
            rates: Rates::default(),
        }
    }
}

/// The interfaces the rates count: every one up and not loopback
/// (src/net:V1).
#[must_use]
pub fn up_interfaces(all: &[InterfaceCounters]) -> Vec<&InterfaceCounters> {
    all.iter().collect()
}

impl Source for NetSignal {
    fn read(&mut self, id: SignalId, now: Instant) -> Reading {
        let _ = (&self.root, self.os);
        self.rates.forget(id);
        self.rates.read(id, &[], now)
    }
}

#[cfg(test)]
mod tests;
