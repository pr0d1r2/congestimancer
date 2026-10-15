//! The arithmetic of the memory gates beyond the raw reading: available
//! memory per CPU (`--mem`, src/mem:V2) and memory not yet reserved
//! (`--needs-mem`, src/mem:V3).

use crate::units::Bytes;

/// `--mem`: available memory per CPU, `available / cpus`, rounded down;
/// `None` for no CPUs, never a division by zero (src/mem:V2).
#[must_use]
pub fn mem_per_cpu(available: Bytes, cpus: u64) -> Option<Bytes> {
    available.0.checked_div(cpus).map(Bytes)
}

/// `--needs-mem`: available memory less what running cgmr jobs reserved,
/// never below zero (src/mem:V3). `reserved` is the ledger's total,
/// [`reserved_total`](crate::slot::reserved_total), read by the caller
/// that holds the lock dir.
#[must_use]
pub const fn mem_unreserved(available: Bytes, reserved: Bytes) -> Bytes {
    Bytes(available.0.saturating_sub(reserved.0))
}

#[cfg(test)]
mod tests;
