//! How many jobs the CPUs and the memory hold now (src/advise:T1, V1,
//! V2).

use crate::units::{Bytes, Decimal};

use crate::mem::mem_unreserved;

/// Thousandths in one, the scale of a [`Decimal`].
const SCALE: u64 = 1_000;

/// CPU fit: `floor(N * L - load)`, never below 0 (src/advise:V1). `N`
/// is the logical CPU count, `L` the `--load` threshold per CPU, `load`
/// the whole machine's load average; the product saturates.
#[must_use]
pub const fn cpu_fit(cpus: u64, load_limit: Decimal, load: Decimal) -> u64 {
    let room = cpus
        .saturating_mul(load_limit.thousandths())
        .saturating_sub(load.thousandths());
    room / SCALE
}

/// Memory fit: `floor((available - reserved) / M)` (src/advise:V2), the
/// difference never below 0; a need of 0 bytes bounds nothing
/// (`u64::MAX`).
#[must_use]
pub const fn mem_fit(available: Bytes, reserved: Bytes, needs: Bytes) -> u64 {
    match mem_unreserved(available, reserved).0.checked_div(needs.0) {
        Some(fit) => fit,
        None => u64::MAX,
    }
}

#[cfg(test)]
mod tests;
