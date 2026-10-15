//! [`decide`]: how many jobs to start, what bounds them, what to do
//! (src/advise:T2, V3, V4, V5).

use std::fmt;

use super::{cpu_fit, mem_fit};
use crate::gate::Checked;
use crate::units::{Bytes, Decimal};

/// One GiB, the default `--needs-mem` (src/advise:V2).
const GIB: u64 = 1 << 30;

/// The default `--load` per CPU, 1.0 (src/advise:V1).
const LOAD_ONE: Decimal = Decimal::from_thousandths(1_000);

/// What `advise` sizes a batch with: the machine's facts and the
/// switches that set the job's size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Request {
    /// `N`: the logical CPUs, from [`cpu_count`](crate::cpu::cpu_count).
    pub cpus: u64,
    /// `L`: `--load`, the load per CPU the batch may bring (default 1.0).
    pub load_limit: Decimal,
    /// `M`: `--needs-mem`, the memory one job needs (default 1GiB).
    pub needs_mem: Bytes,
    /// `J`: `--jobs`, the parallelism asked for, if any.
    pub jobs: Option<u64>,
    /// Memory running cgmr jobs already reserved, from
    /// [`reserved_total`](crate::slot::reserved_total).
    pub reserved: Bytes,
}

impl Request {
    /// A request on `cpus` CPUs with every default: `--load 1`,
    /// `--needs-mem 1GiB`, no `--jobs`, nothing reserved.
    #[must_use]
    pub const fn new(cpus: u64) -> Self {
        Self {
            cpus,
            load_limit: LOAD_ONE,
            needs_mem: Bytes(GIB),
            jobs: None,
            reserved: Bytes(0),
        }
    }

    /// The same request with `--load L`.
    #[must_use]
    pub const fn with_load_limit(mut self, limit: Decimal) -> Self {
        self.load_limit = limit;
        self
    }

    /// The same request with `--needs-mem M`.
    #[must_use]
    pub const fn with_needs_mem(mut self, needs: Bytes) -> Self {
        self.needs_mem = needs;
        self
    }

    /// The same request with `--jobs J`.
    #[must_use]
    pub const fn with_jobs(mut self, jobs: u64) -> Self {
        self.jobs = Some(jobs);
        self
    }

    /// The same request with `reserved` bytes held by running jobs.
    #[must_use]
    pub const fn with_reserved(mut self, reserved: Bytes) -> Self {
        self.reserved = reserved;
        self
    }
}

/// The resource that bounds the advice (src/advise:V5).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Limit {
    /// The CPU fit.
    Cpu,
    /// The memory fit.
    Mem,
    /// `--jobs J` itself.
    Jobs,
    /// A failing gate, by its switch name without `--`.
    Gate(String),
}

/// `cpu`, `mem`, `jobs`, or the gate's switch name.
impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f;
        Ok(())
    }
}

/// What to do now (src/advise:V4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Start the jobs asked for (or as many as fit, with no `--jobs`).
    Run,
    /// Start fewer than `--jobs` asked for.
    Reduce,
    /// Start nothing yet.
    Wait,
}

/// `run`, `reduce` or `wait`.
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f;
        Ok(())
    }
}

/// The advice: how many jobs, what bounds them, what to do, and the
/// gates that block.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Advice {
    /// How many parallel jobs fit now.
    pub jobs: u64,
    /// The resource that bounds `jobs`.
    pub limit: Limit,
    /// What to do.
    pub action: Action,
    /// The requested gates that did not pass, with value and threshold.
    pub blocking: Vec<Checked>,
}

/// Decides from the machine's whole `load` average and its `available`
/// memory, and the verdicts of every other requested gate
/// (src/advise:V3, V4, V5).
#[must_use]
pub fn decide(
    request: &Request,
    load: Decimal,
    available: Bytes,
    gates: &[Checked],
) -> Advice {
    let _ = (cpu_fit, mem_fit, request, load, available, gates);
    Advice {
        jobs: 0,
        limit: Limit::Cpu,
        action: Action::Wait,
        blocking: Vec::new(),
    }
}

#[cfg(test)]
mod tests;
