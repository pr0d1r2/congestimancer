//! [`CgroupSignal`]: the signals a cgroup v2 answers from its own files
//! (src/cgroup:T3, T4, V2, V4).

use super::{
    Cgroup, memory_available, parse_cpu_stat, parse_memory_current,
    parse_memory_max,
};
use crate::signal::{
    Counter, Measure, Reason, Root, Sample, Signal, SignalId, parse_psi,
};
use crate::sys::read_text;
use crate::units::Bytes;

/// The signals one cgroup answers from its own files, read under a
/// [`Root`]; wrap it in [`Sampled`](crate::signal::Sampled) for a
/// [`Source`](crate::signal::Source).
///
/// `--cpu-throttle` reads `cpu.stat` `throttled_usec` as busy time, so it
/// reads as the share of the last interval the quota stopped the cgroup
/// (src/cgroup:V4). Pressure gates read `avg10` of its `*.pressure`
/// files; `--mem` reads the least memory any limited cgroup of its
/// lineage has left (src/cgroup:V2). Any other signal is unsupported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CgroupSignal {
    /// The directory that stands for `/`.
    root: Root,
    /// The cgroup read.
    cgroup: Cgroup,
}

impl CgroupSignal {
    /// Reads `cgroup` under `root`.
    #[must_use]
    pub const fn new(root: Root, cgroup: Cgroup) -> Self {
        Self { root, cgroup }
    }

    /// The cgroup read.
    #[must_use]
    pub const fn cgroup(&self) -> &Cgroup {
        &self.cgroup
    }
}

impl CgroupSignal {
    /// The text of `cgroup`'s file `name`, or why it is unreadable.
    fn read_in(&self, cgroup: &Cgroup, name: &str) -> Result<String, Reason> {
        read_text(&self.root, &cgroup.file(name)).map_err(reason)
    }

    /// The text of the cgroup's file `name`, or why it is unreadable.
    fn read(&self, name: &str) -> Result<String, Reason> {
        self.read_in(&self.cgroup, name)
    }

    /// A pressure gate: `avg10` of the `some` or `full` line of `name`.
    fn pressure(&self, name: &str, full: bool) -> Result<Sample, Reason> {
        let psi = parse_psi(&self.read(name)?).map_err(reason)?;
        let line = if full { psi.full } else { Some(psi.some) };
        let line = line.ok_or_else(|| {
            let file = self.cgroup.file(name);
            Reason::from(format!("{file} has no full line"))
        })?;
        Ok(Sample::Gauge(Measure::Percent(line.avg10)))
    }

    /// Memory left: the least any limited cgroup of the lineage has left
    /// (src/cgroup:V2).
    fn memory(&self) -> Result<Sample, Reason> {
        let mut least: Option<Bytes> = None;
        for cgroup in self.cgroup.lineage() {
            if let Some(left) = self.memory_left(&cgroup)? {
                least = Some(least.map_or(left, |least| least.min(left)));
            }
        }
        let least = least.ok_or_else(|| {
            let path = self.cgroup.path();
            Reason::from(format!("{path} has no memory limit"))
        })?;
        Ok(Sample::Gauge(Measure::Bytes(least)))
    }

    /// What `cgroup` has left of its own memory limit, if it has one; an
    /// unreadable `memory.max` (the root, no memory controller) is none.
    fn memory_left(&self, cgroup: &Cgroup) -> Result<Option<Bytes>, Reason> {
        let Ok(max) = self.read_in(cgroup, "memory.max") else {
            return Ok(None);
        };
        let Some(max) = parse_memory_max(&max).map_err(reason)? else {
            return Ok(None);
        };
        let current = self.read_in(cgroup, "memory.current")?;
        let current = parse_memory_current(&current).map_err(reason)?;
        Ok(Some(memory_available(max, current, None)))
    }

    /// `--cpu-throttle`: time the quota stopped the cgroup, as busy time.
    fn throttle(&self) -> Result<Sample, Reason> {
        let stat = parse_cpu_stat(&self.read("cpu.stat")?).map_err(reason)?;
        let throttling = stat.throttling.ok_or_else(|| {
            let path = self.cgroup.path();
            Reason::from(format!("{path} has no cpu controller to throttle it"))
        })?;
        let busy = throttling.throttled;
        Ok(Sample::Counter(Counter::Busy { busy }))
    }
}

impl Signal for CgroupSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let sample = match id {
            SignalId::CpuThrottle => self.throttle(),
            SignalId::CpuPressure => self.pressure("cpu.pressure", false),
            SignalId::MemPressure => self.pressure("memory.pressure", false),
            SignalId::MemPressureFull => self.pressure("memory.pressure", true),
            SignalId::IoPressure => self.pressure("io.pressure", false),
            SignalId::IoPressureFull => self.pressure("io.pressure", true),
            SignalId::Mem => self.memory(),
            _ => Err(Reason::from(format!("{id} is not a cgroup signal"))),
        };
        sample.unwrap_or_else(Sample::Unsupported)
    }
}

/// An error as the reason a signal has no value.
fn reason(err: impl ToString) -> Reason {
    Reason::from(err.to_string())
}

#[cfg(test)]
mod tests;
