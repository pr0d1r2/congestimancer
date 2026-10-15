//! The logical CPU count `N` and the per-CPU arithmetic of `--load` and
//! `--run-queue` (src/cpu:T1, V1, V3, V7).

use std::io;
use std::num::NonZeroUsize;

use crate::units::Decimal;

/// `N`: the logical CPUs this process may run on, from
/// [`std::thread::available_parallelism`]; at least 1, also when the
/// count cannot be read (src/cpu:V7).
#[must_use]
pub fn cpu_count() -> u64 {
    cpus_from(std::thread::available_parallelism())
}

/// The CPU count a read of `available_parallelism` gives; 1 when the
/// read failed.
fn cpus_from(read: io::Result<NonZeroUsize>) -> u64 {
    read.ok()
        .and_then(|n| u64::try_from(n.get()).ok())
        .unwrap_or(1)
}

/// `value / cpus`, rounded down to thousandths: load or running tasks
/// per CPU (`--load`, `--run-queue`; src/cpu:V1, V3). `None` for no CPUs,
/// never a division by zero (src/cpu:V7).
#[must_use]
pub fn load_per_cpu(value: Decimal, cpus: u64) -> Option<Decimal> {
    value
        .thousandths()
        .checked_div(cpus)
        .map(Decimal::from_thousandths)
}

#[cfg(test)]
mod tests;
