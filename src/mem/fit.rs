//! The arithmetic of the memory gates beyond the raw reading: available
//! memory per CPU (`--mem`, src/mem:V2).

use crate::units::Bytes;

/// `--mem`: available memory per CPU, `available / cpus`, rounded down;
/// `None` for no CPUs, never a division by zero (src/mem:V2).
#[must_use]
pub const fn mem_per_cpu(available: Bytes, cpus: u64) -> Option<Bytes> {
    let _ = (available, cpus);
    None
}

#[cfg(test)]
mod tests;
