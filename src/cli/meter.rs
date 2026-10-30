//! [`PeakMeter`]: watches a command's memory while it runs and keeps the
//! peak for its key when it ends (src/slot:V13). On Linux the reading is
//! the job's cgroup's unreclaimable memory when the cgroup holds the job
//! alone (src/cgroup:V7), else the resident memory of its process tree.

use std::time::{Duration, Instant};

use super::learn::Learning;
use crate::cgroup::{Cgroup, parse_proc_cgroup, parse_unreclaimable};
use crate::launch::Meter;
use crate::signal::Root;
use crate::slot::record_peak;
use crate::sys::{Snapshot, read_text};
use crate::units::Bytes;

/// The shortest wait between two samples.
const FLOOR: Duration = Duration::from_secs(1);

/// The longest wait between two samples (a 10 s cadence for long jobs).
const CEILING: Duration = Duration::from_secs(10);

/// How long to wait after a sample at `elapsed` into the run: half of
/// it, from a second to ten, so a short job is caught early and a long
/// one is not read often.
pub(crate) fn gap(elapsed: Duration) -> Duration {
    elapsed.mul_f64(0.5).clamp(FLOOR, CEILING)
}

/// Whether `procs` (a `cgroup.procs`) lists only members of `pids`, and
/// someone: the cgroup then holds the job alone.
pub(crate) fn exclusive(procs: &str, pids: &[u32]) -> bool {
    let listed: Vec<u32> = procs
        .split_whitespace()
        .filter_map(|w| w.parse().ok())
        .collect();
    !listed.is_empty() && listed.iter().all(|p| pids.contains(p))
}

/// The memory the job holds: its cgroup's unreclaimable memory when that
/// cgroup is its own, else `rss`, the resident memory of its tree.
pub(crate) fn job_memory(
    root: &Root,
    leader: u32,
    pids: &[u32],
    rss: u64,
) -> u64 {
    cgroup_memory(root, leader, pids).unwrap_or(rss)
}

/// The unreclaimable memory of the cgroup `leader` is in, if only the job
/// is in it and its files can be read.
fn cgroup_memory(root: &Root, leader: u32, pids: &[u32]) -> Option<u64> {
    let text = read_text(root, &format!("/proc/{leader}/cgroup")).ok()?;
    let cgroup = Cgroup::new(&parse_proc_cgroup(&text).ok()?);
    let procs = read_text(root, &cgroup.file("cgroup.procs")).ok()?;
    if !exclusive(&procs, pids) {
        return None;
    }
    let tree = read_text(root, &cgroup.file("cgroup.stat")).ok()?;
    if !childless(&tree) {
        return None;
    }
    let stat = read_text(root, &cgroup.file("memory.stat")).ok()?;
    parse_unreclaimable(&stat).ok().map(|b| b.0)
}

/// Whether `cgroup.stat` counts no descendant cgroups, live or dying:
/// `memory.stat` also charges their memory to the cgroup.
fn childless(stat: &str) -> bool {
    ["nr_descendants", "nr_dying_descendants"]
        .iter()
        .all(|key| stat.lines().any(|line| line == format!("{key} 0")))
}

/// Keeps the highest memory a command is seen to hold.
pub(crate) struct PeakMeter {
    /// Where the peak goes.
    learning: Learning,
    /// Where the kernel's files are read from.
    root: Root,
    /// The process view the tree is read through.
    snapshot: Snapshot,
    /// The first sample's instant.
    started: Option<Instant>,
    /// The last sample's instant.
    last: Option<Instant>,
    /// The highest reading so far.
    peak: u64,
}

impl PeakMeter {
    /// A meter for `learning`, reading files under `root`.
    pub(crate) fn new(learning: Learning, root: Root) -> Self {
        Self {
            learning,
            root,
            snapshot: Snapshot::new(),
            started: None,
            last: None,
            peak: 0,
        }
    }

    /// The highest reading so far.
    pub(crate) const fn peak(&self) -> Bytes {
        Bytes(self.peak)
    }

    /// Whether a sample at `now` is due.
    fn due(&mut self, now: Instant) -> bool {
        let started = *self.started.get_or_insert(now);
        let wait = gap(now.saturating_duration_since(started));
        self.last
            .is_none_or(|last| now.saturating_duration_since(last) >= wait)
    }
}

impl Meter for PeakMeter {
    fn sample(&mut self, leader: Option<u32>, now: Instant) {
        let Some(leader) = leader.filter(|_| self.due(now)) else {
            return;
        };
        self.last = Some(now);
        let tree = self.snapshot.tree_memory(leader);
        if tree.pids.is_empty() {
            return;
        }
        let held = job_memory(&self.root, leader, &tree.pids, tree.rss);
        self.peak = self.peak.max(held);
    }

    fn finish(&mut self) {
        // A ledger that cannot be written costs the next start a guess,
        // not this run its exit code.
        let _ =
            record_peak(self.learning.dir(), self.learning.name(), self.peak());
    }
}

#[cfg(test)]
mod tests;
