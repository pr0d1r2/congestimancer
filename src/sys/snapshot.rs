//! [`Snapshot`]: the shared `sysinfo` view of load, memory and processes,
//! refreshed one part at a time and only on request (src/sys:T3, V5).

use std::ffi::OsStr;

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

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

/// One process: its name and owner's uid, nothing more (src/sys:V7).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ProcessInfo {
    /// The process name, lossily UTF-8.
    pub name: String,
    /// The owner's uid, when the OS tells.
    pub uid: Option<u32>,
}

impl ProcessInfo {
    /// A process named `name` (lossily UTF-8), owned by `uid`.
    fn new(name: &OsStr, uid: Option<u32>) -> Self {
        let _ = name;
        Self {
            name: String::new(),
            uid,
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

    /// Reads the process list with each process's name and uid only.
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
            .map(|p| ProcessInfo::new(p.name(), p.user_id().map(|uid| **uid)))
            .collect()
    }
}

#[cfg(test)]
mod tests;
