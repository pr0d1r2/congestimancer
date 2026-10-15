//! [`NetSignal`]: `--net-rx`, `--net-tx` and `--net` over the chosen
//! interfaces (src/net:T2, T4, V1, V2).

use std::time::Instant;

use super::dev::linux_interfaces;
use super::rates::Rates;
use crate::signal::{Reading, Reason, Root, SignalId, Source};
use crate::sys::{InterfaceCounters, interface_counters};

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
    all.iter().filter(|i| i.up && !i.loopback).collect()
}

impl NetSignal {
    /// Every interface and its totals, or why there are none.
    fn interfaces(&self) -> Result<Vec<InterfaceCounters>, Reason> {
        let found = match self.os {
            Os::Linux => linux_interfaces(&self.root),
            Os::Macos => interface_counters(),
        };
        found.map_err(|err| Reason::from(err.to_string()))
    }
}

impl Source for NetSignal {
    fn read(&mut self, id: SignalId, now: Instant) -> Reading {
        if !matches!(id, SignalId::NetRx | SignalId::NetTx | SignalId::Net) {
            return Reading::unsupported(format!("{id} is not a net rate"));
        }
        match self.interfaces() {
            Ok(all) => self.rates.read(id, &up_interfaces(&all), now),
            Err(reason) => {
                self.rates.forget(id);
                Reading::Unsupported { reason }
            }
        }
    }
}

#[cfg(test)]
mod tests;
