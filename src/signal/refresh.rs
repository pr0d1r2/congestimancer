//! [`SignalSet`] and [`refresh`]: read only the signals the requested
//! gates use (src/signal:T6, V3).

use std::time::Instant;

use super::{Reading, SignalId, Source};

/// A set of signals: the ones the requested gates read.
///
/// Built from the gate switches on the command line
/// ([`SignalSet::from_switches`]); [`refresh`] reads these and no others,
/// so `cgmr --load 2` never walks block devices or loads NVML
/// (src/signal:V3).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SignalSet(u64);

impl SignalSet {
    /// The empty set.
    #[must_use]
    pub const fn new() -> Self {
        Self(0)
    }

    /// The signals the given gate switches read (names without `--`);
    /// switches that read no signal are skipped.
    #[must_use]
    pub fn from_switches<'a>(
        switches: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        let _ = switches.into_iter();
        Self::new()
    }

    /// Adds `id`; `true` if it was not in the set yet.
    pub fn insert(&mut self, id: SignalId) -> bool {
        let _ = id;
        false
    }

    /// Whether `id` is in the set.
    #[must_use]
    pub fn contains(self, id: SignalId) -> bool {
        let _ = id;
        false
    }

    /// How many signals are in the set.
    #[must_use]
    pub const fn len(self) -> usize {
        0
    }

    /// Whether the set is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The signals in the set, in [`SignalId::ALL`] order.
    pub fn iter(self) -> impl Iterator<Item = SignalId> {
        SignalId::ALL
            .into_iter()
            .filter(move |&id| self.contains(id))
    }

    /// Whether any signal in the set is a rate or interval percent, so a
    /// one-shot check takes a second sample after the interval
    /// (src/signal:V1).
    #[must_use]
    pub fn needs_two_samples(self) -> bool {
        false
    }
}

impl FromIterator<SignalId> for SignalSet {
    fn from_iter<I: IntoIterator<Item = SignalId>>(ids: I) -> Self {
        let mut set = Self::new();
        set.extend(ids);
        set
    }
}

impl Extend<SignalId> for SignalSet {
    fn extend<I: IntoIterator<Item = SignalId>>(&mut self, ids: I) {
        for id in ids {
            self.insert(id);
        }
    }
}

/// Reads every signal in `wanted` from `source` at `now`, in
/// [`SignalId::ALL`] order, and touches no other (src/signal:V3).
pub fn refresh<S: Source + ?Sized>(
    source: &mut S,
    wanted: SignalSet,
    now: Instant,
) -> Vec<(SignalId, Reading)> {
    let _ = (source, wanted, now);
    Vec::new()
}

#[cfg(test)]
mod tests;
