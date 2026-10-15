//! [`ProcsSignal`]: `--max-procs` as a [`Signal`] over a process table
//! (src/procs:T3, T4).

use super::{Glob, Proc, count_by_name};
use crate::signal::{Measure, Reason, Sample, Signal, SignalId};
use crate::sys::Snapshot;

/// Where a [`ProcsSignal`] lists processes: the machine, or a test table.
pub trait ProcessTable {
    /// Every process now.
    fn processes(&mut self) -> Vec<Proc>;
}

/// The machine's process table, through `crate::sys` (sysinfo).
#[derive(Debug, Default)]
pub struct LiveTable {
    /// The shared sysinfo view; only its process part is refreshed.
    snapshot: Snapshot,
}

impl ProcessTable for LiveTable {
    fn processes(&mut self) -> Vec<Proc> {
        self.snapshot.refresh_processes();
        let procs = self.snapshot.processes();
        procs
            .into_iter()
            .map(|p| Proc::new(p.name, p.uid))
            .collect()
    }
}

/// What a [`ProcsSignal`] counts processes by.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ProcMatch {
    /// Process names matching the glob (`--max-procs`).
    Name(Glob),
}

/// Counts the processes one switch's pattern matches, leaving out cgmr
/// and its ancestors (src/procs:V3); one per `--max-procs` value.
#[derive(Debug)]
pub struct ProcsSignal<T = LiveTable> {
    /// Where processes are listed.
    table: T,
    /// What is counted.
    by: ProcMatch,
    /// cgmr's own pid.
    me: u32,
}

impl ProcsSignal {
    /// Counts `by` on this machine, as this process.
    #[must_use]
    pub fn live(by: ProcMatch) -> Self {
        Self::over(LiveTable::default(), by, std::process::id())
    }
}

impl<T: ProcessTable> ProcsSignal<T> {
    /// Counts `by` over `table`, as process `me`.
    pub const fn over(table: T, by: ProcMatch, me: u32) -> Self {
        Self { table, by, me }
    }
}

impl<T: ProcessTable> Signal for ProcsSignal<T> {
    fn sample(&mut self, id: SignalId) -> Sample {
        let count = match (id, &self.by) {
            (SignalId::MaxProcs, ProcMatch::Name(glob)) => {
                count_by_name(&self.table.processes(), glob, self.me)
            }
            _ => {
                let why = format!("{id} is not this process gate's signal");
                return Sample::Unsupported(Reason::from(why));
            }
        };
        Sample::Gauge(Measure::Count(count))
    }
}

#[cfg(test)]
mod tests;
