//! [`MemSignal`]: the memory signals, from Linux `/proc` under a
//! [`Root`] or macOS sysctls and mach counters (src/mem:T1, T3-T7).

use std::io;

use super::parse::{MEMINFO, SMAPS, VMSTAT};
use super::{
    SwapPages, parse_meminfo, parse_page_size, parse_vmstat, pressure_level,
};
use crate::signal::{
    Counter, Level, Measure, Reading, Reason, Root, Sample, Signal, SignalId,
    parse_psi,
};
use crate::sys::{Snapshot, VmStats, read_text, sysctl_int, vm_stats};
use crate::units::{Bytes, Percent};

/// Linux memory pressure stall information.
const PRESSURE: &str = "/proc/pressure/memory";

/// macOS memory pressure level: 1 normal, 2 warning, 4 critical.
const PRESSURE_LEVEL: &str = "kern.memorystatus_vm_pressure_level";

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
    /// The page size, once read.
    page_size: Option<u64>,
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
            page_size: None,
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

    /// `--mem-pressure` (`some avg10`) or, when `full`,
    /// `--mem-pressure-full` (`full avg10`); Linux only (src/mem:V4).
    fn pressure(&self, full: bool) -> Result<Percent, Reason> {
        if self.os == Os::Macos {
            return Err(Reason::from("no PSI on macOS; use --mem-level"));
        }
        let text = read_text(&self.root, PRESSURE).map_err(reason)?;
        let psi = parse_psi(&text).map_err(reason)?;
        let line = if full { psi.full } else { Some(psi.some) };
        line.map(|line| line.avg10).ok_or_else(|| {
            Reason::from(format!("{PRESSURE} has no `full` line"))
        })
    }

    /// `--mem-level`: the kernel's own pressure level; macOS only
    /// (src/mem:V5).
    fn level(&self) -> Result<Level, Reason> {
        if self.os == Os::Linux {
            return Err(Reason::from(
                "no memory pressure level on Linux; use --mem-pressure",
            ));
        }
        level_from(sysctl_int(PRESSURE_LEVEL))
    }

    /// `--swap-rate`: bytes swapped in plus out so far, a counter the
    /// sampler turns into bytes per second (src/mem:V6).
    fn swapped(&mut self) -> Result<u64, Reason> {
        match self.os {
            Os::Linux => {
                let text = read_text(&self.root, VMSTAT).map_err(reason)?;
                let pages = parse_vmstat(&text).map_err(reason)?;
                let page_size = self.linux_page_size()?;
                swap_bytes(pages, page_size)
            }
            Os::Macos => macos_swapped(vm_stats(), sysctl_int("hw.pagesize")),
        }
    }

    /// `--dirty`: `Dirty` plus `Writeback`; Linux only (src/mem:V7).
    fn dirty(&self) -> Result<Bytes, Reason> {
        if self.os == Os::Macos {
            return Err(Reason::from("no dirty page count on macOS"));
        }
        let text = read_text(&self.root, MEMINFO).map_err(reason)?;
        let pages = parse_meminfo(&text).map_err(reason)?;
        pages
            .backlog()
            .ok_or_else(|| Reason::from("dirty bytes past 64 bits"))
    }

    /// The Linux base page size from `/proc/self/smaps`, read once.
    fn linux_page_size(&mut self) -> Result<u64, Reason> {
        if let Some(size) = self.page_size {
            return Ok(size);
        }
        let text = read_text(&self.root, SMAPS).map_err(reason)?;
        let size = parse_page_size(&text).map_err(reason)?;
        self.page_size = Some(size);
        Ok(size)
    }
}

impl MemSignal {
    /// The share of memory macOS counts as available
    /// (`kern.memorystatus_level`), shown by `status`; Linux has none
    /// (src/mem:V8).
    #[must_use]
    pub fn status_level(&self) -> Reading {
        status_from(Ok(0))
    }
}

impl Signal for MemSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let sample = match id {
            SignalId::SwapRate => self
                .swapped()
                .map(|total| Sample::Counter(Counter::Rate { total })),
            _ => self.gauge(id).map(Sample::Gauge),
        };
        sample.unwrap_or_else(Sample::Unsupported)
    }
}

impl MemSignal {
    /// The value of a memory signal that stands alone.
    fn gauge(&mut self, id: SignalId) -> Result<Measure, Reason> {
        match id {
            SignalId::Mem => Ok(Measure::Bytes(self.available())),
            SignalId::MemPressure => self.pressure(false).map(Measure::Percent),
            SignalId::MemPressureFull => {
                self.pressure(true).map(Measure::Percent)
            }
            SignalId::MemLevel => self.level().map(Measure::Level),
            SignalId::Dirty => self.dirty().map(Measure::Bytes),
            _ => Err(Reason::from(format!("{id} is not a mem signal"))),
        }
    }
}

/// `pages` in bytes of `page_size`; past 64 bits is unsupported, never
/// wrapped (src/mem:V6).
fn swap_bytes(pages: SwapPages, page_size: u64) -> Result<u64, Reason> {
    pages
        .bytes(page_size)
        .ok_or_else(|| Reason::from("swapped bytes past 64 bits"))
}

/// The macOS swapped bytes from `host_statistics64` swapins and
/// swapouts and the `hw.pagesize` sysctl (src/mem:V6).
fn macos_swapped(
    stats: io::Result<VmStats>,
    page_size: io::Result<i64>,
) -> Result<u64, Reason> {
    let stats = stats.map_err(reason)?;
    let raw = page_size.map_err(reason)?;
    let page_size = u64::try_from(raw)
        .map_err(|_| Reason::from(format!("hw.pagesize: negative {raw}")))?;
    let pages = SwapPages {
        pages_in: stats.swapins,
        pages_out: stats.swapouts,
    };
    swap_bytes(pages, page_size)
}

/// The percent a read of `kern.memorystatus_level` gives; a failed read
/// or a number outside 0 to 100 is unsupported with its reason.
fn status_from(raw: io::Result<i64>) -> Reading {
    let _ = raw;
    Reading::Warming
}

/// The level a read of `kern.memorystatus_vm_pressure_level` gives; a
/// failed read or an unknown number is unsupported with its reason.
fn level_from(raw: io::Result<i64>) -> Result<Level, Reason> {
    let raw = raw.map_err(reason)?;
    pressure_level(raw).ok_or_else(|| {
        Reason::from(format!("{PRESSURE_LEVEL}: unknown level {raw}"))
    })
}

/// An error as the reason a signal has no value.
fn reason(err: impl ToString) -> Reason {
    Reason::from(err.to_string())
}

#[cfg(test)]
mod tests;
