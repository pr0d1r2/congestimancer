//! [`Rates`]: `--net-rx`, `--net-tx` and `--net` from two samples of the
//! chosen interfaces' totals (src/net:T2, T4, V2, V5).

use std::time::Instant;

use crate::signal::{Reading, Sampler, SignalId};
use crate::sys::InterfaceCounters;

/// The byte totals per signal over the chosen interfaces, through one
/// [`Sampler`].
///
/// The sampler sees the sum; this sees each interface. When the set of
/// interfaces changes, or any one's total went back (a 32-bit `if_data`
/// wrap, a driver reset), the base is dropped and the signal reads
/// `Warming` again: never a rate made of a jump (src/net:V5).
#[derive(Debug, Default)]
pub(crate) struct Rates {
    /// The sums turned into rates.
    sampler: Sampler,
}

impl Rates {
    /// Reads `id` over `ifaces` at `now`.
    pub(crate) fn read(
        &mut self,
        _id: SignalId,
        _ifaces: &[&InterfaceCounters],
        _now: Instant,
    ) -> Reading {
        let _ = &self.sampler;
        Reading::Warming
    }

    /// Drops the base of `id`: its next read warms up again.
    pub(crate) fn forget(&mut self, _id: SignalId) {}
}

#[cfg(test)]
mod tests;
