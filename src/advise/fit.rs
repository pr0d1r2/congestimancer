//! How many jobs the CPUs and the memory hold now (src/advise:T1, V1,
//! V2).

use crate::units::{Bytes, Decimal};

/// CPU fit: `floor(N * L - load)`, never below 0 (src/advise:V1).
#[must_use]
pub const fn cpu_fit(cpus: u64, load_limit: Decimal, load: Decimal) -> u64 {
    let _ = (cpus, load_limit, load);
    0
}

/// Memory fit: `floor((available - reserved) / M)` (src/advise:V2).
#[must_use]
pub const fn mem_fit(available: Bytes, reserved: Bytes, needs: Bytes) -> u64 {
    let _ = (available, reserved, needs);
    0
}

#[cfg(test)]
mod tests;
