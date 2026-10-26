//! [`Snapshot`]: the shared `sysinfo` view of load, memory and processes,
//! refreshed one part at a time and only on request (src/sys:T3, V5).

use std::ffi::OsStr;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// The 1, 5 and 15 minute load averages.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[non_exhaustive]
pub struct LoadAverage {
    /// Over the last minute.
    pub one: f64,
    /// Over the last five minutes.
    pub five: f64,
    /// Over the last fifteen minutes.
    pub fifteen: f64,
}

/// Memory and swap, in bytes, as of the last [`Snapshot::refresh_memory`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Memory {
    /// Physical memory.
    pub total: u64,
    /// Memory a new process can get without swapping.
    pub available: u64,
    /// Swap space.
    pub swap_total: u64,
    /// Swap space in use.
    pub swap_used: u64,
}

/// One process: its name, owner's uid and lineage, nothing more
/// (src/sys:V7).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ProcessInfo {
    /// The process name, lossily UTF-8.
    pub name: String,
    /// The owner's uid, when the OS tells.
    pub uid: Option<u32>,
    /// The process id.
    pub pid: u32,
    /// The parent's process id, when it has one.
    pub parent: Option<u32>,
}

impl ProcessInfo {
    /// A process named `name` (lossily UTF-8), owned by `uid`, with id
    /// `pid` and parent `parent`.
    fn new(
        name: &OsStr,
        uid: Option<u32>,
        pid: u32,
        parent: Option<u32>,
    ) -> Self {
        Self {
            name: name.to_string_lossy().into_owned(),
            uid,
            pid,
            parent,
        }
    }
}

/// One `sysinfo` view of the machine. Nothing is read until a `refresh_*`
/// asks for that part, and each refresh reads only its own part
/// (src/sys:V5, src/signal:V3).
#[derive(Debug, Default)]
pub struct Snapshot {
    system: System,
}

impl Snapshot {
    /// A snapshot with nothing read yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The load averages, read now (no refresh needed).
    #[must_use]
    pub fn load_average() -> LoadAverage {
        let load = System::load_average();
        LoadAverage {
            one: load.one,
            five: load.five,
            fifteen: load.fifteen,
        }
    }

    /// Reads memory and swap, and nothing else.
    pub fn refresh_memory(&mut self) {
        self.system.refresh_memory();
    }

    /// Memory and swap as of the last [`Self::refresh_memory`]; all zero
    /// before the first.
    #[must_use]
    pub fn memory(&self) -> Memory {
        Memory {
            total: self.system.total_memory(),
            available: self.system.available_memory(),
            swap_total: self.system.total_swap(),
            swap_used: self.system.used_swap(),
        }
    }

    /// Reads the process list with each process's name, uid, pid and
    /// parent only.
    pub fn refresh_processes(&mut self) {
        let kind =
            ProcessRefreshKind::nothing().with_user(UpdateKind::OnlyIfNotSet);
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            kind,
        );
    }

    /// Every process as of the last [`Self::refresh_processes`]; empty
    /// before the first.
    #[must_use]
    pub fn processes(&self) -> Vec<ProcessInfo> {
        self.system
            .processes()
            .values()
            .map(|p| {
                let uid = p.user_id().map(|uid| **uid);
                let parent = p.parent().map(Pid::as_u32);
                ProcessInfo::new(p.name(), uid, p.pid().as_u32(), parent)
            })
            .collect()
    }

    /// The process `root` and everything below it, with the resident
    /// memory they hold together; empty and zero when `root` is gone.
    /// Reads every process's memory, so ask sparingly.
    pub fn tree_memory(&mut self, root: u32) -> TreeMemory {
        let kind = ProcessRefreshKind::nothing().with_memory();
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            kind,
        );
        tree_of(&self.resident(), root)
    }

    /// Every process, not its threads, as (pid, parent, resident bytes),
    /// as of the last refresh.
    fn resident(&self) -> Vec<Resident> {
        let processes = self.system.processes().values();
        processes
            .filter(|p| p.thread_kind().is_none())
            .map(|p| {
                let parent = p.parent().map(Pid::as_u32);
                (p.pid().as_u32(), parent, p.memory())
            })
            .collect()
    }
}

/// A process as (pid, parent, resident bytes).
type Resident = (u32, Option<u32>, u64);

/// A process and its descendants.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct TreeMemory {
    /// Their process ids, `root` first.
    pub pids: Vec<u32>,
    /// The resident memory they hold together, in bytes.
    pub rss: u64,
}

/// `root` and every process below it in `procs` (pid, parent, resident
/// bytes), and what they hold together; nothing when `root` is absent.
fn tree_of(procs: &[Resident], root: u32) -> TreeMemory {
    let pids = family(procs, root);
    let held = procs.iter().filter(|p| pids.contains(&p.0));
    let rss = held.fold(0, |total: u64, p| total.saturating_add(p.2));
    TreeMemory { pids, rss }
}

/// `root` and the ids of everything below it in `procs`, `root` first;
/// none when `root` is absent.
fn family(procs: &[Resident], root: u32) -> Vec<u32> {
    let mut pids = Vec::new();
    if procs.iter().any(|p| p.0 == root) {
        pids.push(root);
    }
    let mut next = 0;
    while let Some(&parent) = pids.get(next) {
        let kids: Vec<u32> = procs
            .iter()
            .filter(|p| p.1 == Some(parent) && !pids.contains(&p.0))
            .map(|p| p.0)
            .collect();
        pids.extend(kids);
        next = next.saturating_add(1);
    }
    pids
}

#[cfg(test)]
mod tests;
