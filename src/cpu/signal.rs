//! [`CpuSignal`]: the CPU signals, from Linux `/proc` under a [`Root`]
//! or macOS `sysinfo` and mach calls (src/cpu:T2-T8).

use std::io;

use super::parse::{LOADAVG, STAT};
use super::{LoadWindow, Loads, ProcStat, parse_loadavg, parse_stat};
use crate::signal::{Measure, Reason, Root, Sample, Signal, SignalId};
use crate::sys::{LoadAverage, PsetLoad, Snapshot, read_text};
use crate::units::{Decimal, parse_decimal};

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
    /// The load average `SignalId::Load` reads.
    window: LoadWindow,
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
        Self {
            root,
            os,
            window: LoadWindow::One,
        }
    }

    /// Reads the load average over `window` (`--load-window`,
    /// src/cpu:V2).
    #[must_use]
    pub const fn with_window(mut self, window: LoadWindow) -> Self {
        self.window = window;
        self
    }
}

impl CpuSignal {
    /// The load averages: Linux `/proc/loadavg`, macOS `sysinfo`
    /// (src/cpu:T2).
    fn loads(&self) -> Result<Loads, Reason> {
        match self.os {
            Os::Linux => {
                let text = read_text(&self.root, LOADAVG).map_err(reason)?;
                parse_loadavg(&text).map_err(reason)
            }
            Os::Macos => loads_from(Snapshot::load_average()),
        }
    }

    /// Linux `/proc/stat` under the root (src/cpu:T4).
    fn stat(&self) -> Result<ProcStat, Reason> {
        let text = read_text(&self.root, STAT).map_err(reason)?;
        parse_stat(&text).map_err(reason)
    }

    /// Tasks running or runnable now (`--run-queue`, src/cpu:V3).
    fn run_queue(&self) -> Result<Decimal, Reason> {
        let _ = run_queue_from;
        let running = self.stat()?.procs_running;
        running
            .checked_mul(1000)
            .map(Decimal::from_thousandths)
            .ok_or_else(|| Reason::from("procs_running past 64 bits"))
    }
}

impl Signal for CpuSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        self.gauge(id)
            .map(Sample::Gauge)
            .unwrap_or_else(Sample::Unsupported)
    }
}

impl CpuSignal {
    /// The value of a CPU signal that stands alone.
    fn gauge(&self, id: SignalId) -> Result<Measure, Reason> {
        match id {
            SignalId::Load => {
                self.loads().map(|l| Measure::Number(l.pick(self.window)))
            }
            SignalId::RunQueue => self.run_queue().map(Measure::Number),
            _ => Err(Reason::from(format!("{id} is not a cpu signal"))),
        }
    }
}

/// The load averages `sysinfo` gives, to thousandths; a negative or
/// non-finite average is unsupported.
fn loads_from(load: LoadAverage) -> Result<Loads, Reason> {
    Ok(Loads {
        one: decimal(load.one)?,
        five: decimal(load.five)?,
        fifteen: decimal(load.fifteen)?,
    })
}

/// `load` rounded to thousandths.
fn decimal(load: f64) -> Result<Decimal, Reason> {
    let bad = || Reason::from(format!("load average {load} is not a load"));
    if !load.is_finite() || load < 0.0 {
        return Err(bad());
    }
    parse_decimal(&format!("{load:.3}")).map_err(|_| bad())
}

/// The macOS run queue a read of the default processor set gives: its
/// `load_average` / 1000 (src/cpu:R1, V3); a failed read or a negative
/// load is unsupported.
fn run_queue_from(read: io::Result<PsetLoad>) -> Result<Decimal, Reason> {
    let _ = read;
    Err(Reason::from("todo"))
}

/// An error as the reason a signal has no value.
fn reason(err: impl ToString) -> Reason {
    Reason::from(err.to_string())
}

#[cfg(test)]
mod tests;
