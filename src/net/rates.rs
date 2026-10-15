//! [`Rates`]: `--net-rx`, `--net-tx` and `--net` from two samples of the
//! chosen interfaces' totals (src/net:T2, T4, V2, V5).

use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

use crate::signal::{Counter, Reading, Sampler, SignalId};
use crate::sys::InterfaceCounters;

/// Received and sent bytes so far, per interface name.
type Totals = BTreeMap<String, (u64, u64)>;

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
    /// Each signal's interfaces at its last read.
    last: HashMap<SignalId, Totals>,
}

impl Rates {
    /// Reads `id` (`NetRx`, `NetTx`, or else both summed) over `ifaces`
    /// at `now`.
    pub(crate) fn read(
        &mut self,
        id: SignalId,
        ifaces: &[&InterfaceCounters],
        now: Instant,
    ) -> Reading {
        let totals: Totals = ifaces
            .iter()
            .map(|i| (i.name.clone(), (i.rx.bytes, i.tx.bytes)))
            .collect();
        let total = sum(id, &totals);
        let broken = self
            .last
            .get(&id)
            .is_some_and(|before| !follows(before, &totals));
        if broken {
            self.sampler.forget(id);
        }
        self.last.insert(id, totals);
        self.sampler.update(id, Counter::Rate { total }, now)
    }

    /// Drops the base of `id`: its next read warms up again.
    pub(crate) fn forget(&mut self, id: SignalId) {
        self.last.remove(&id);
        self.sampler.forget(id);
    }
}

/// Whether `after` grew from `before` on the same interfaces, none
/// going back.
fn follows(before: &Totals, after: &Totals) -> bool {
    before.len() == after.len()
        && before
            .iter()
            .zip(after)
            .all(|((n0, a), (n1, b))| n0 == n1 && b.0 >= a.0 && b.1 >= a.1)
}

/// What `id` counts of `totals`: received, sent, or both. The sum wraps
/// rather than saturates, so the sampler sees it go back and warms up.
fn sum(id: SignalId, totals: &Totals) -> u64 {
    totals.values().fold(0, |acc, &(rx, tx)| match id {
        SignalId::NetRx => acc.wrapping_add(rx),
        SignalId::NetTx => acc.wrapping_add(tx),
        _ => acc.wrapping_add(rx).wrapping_add(tx),
    })
}

#[cfg(test)]
mod tests;
