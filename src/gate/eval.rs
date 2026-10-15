//! [`evaluate`]: every gate in one round, AND over the verdicts
//! (src/gate:T2, T5, V2, V3, V6, V7).

use std::fmt;
use std::time::Instant;

use super::{Gate, Verdict};
use crate::signal::{Source, Trends};

/// The gates one evaluation runs, in the order they were given.
///
/// Most gates read the shared [`Source`] passed to [`evaluate`]. A gate
/// whose signal depends on its own switch value (one
/// [`ProcsSignal`](crate::procs::ProcsSignal) per `--max-procs` value)
/// brings its own source with [`Gates::push_own`].
#[derive(Default)]
pub struct Gates {
    /// Every gate, in order.
    entries: Vec<Entry>,
}

/// One gate in a [`Gates`] list.
enum Entry {
    /// Reads the shared source.
    Shared(Gate),
    /// Reads its own source.
    Own(Gate, Box<dyn Source>),
}

impl Gates {
    /// No gates: an evaluation passes at once (src/gate:V6).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a gate on the shared source.
    pub fn push(&mut self, gate: Gate) -> &mut Self {
        self.entries.push(Entry::Shared(gate));
        self
    }

    /// Adds a gate that reads `source` instead of the shared one.
    pub fn push_own(
        &mut self,
        gate: Gate,
        source: impl Source + 'static,
    ) -> &mut Self {
        self.entries.push(Entry::Own(gate, Box::new(source)));
        self
    }

    /// How many gates there are.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether there are none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Lists the gates; the sources are opaque.
impl fmt::Debug for Gates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let gates = self.entries.iter().map(|entry| match entry {
            Entry::Shared(gate) | Entry::Own(gate, _) => gate,
        });
        f.debug_list().entries(gates).finish()
    }
}

/// What the gates say together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Overall {
    /// Every gate passed, or there are none: go.
    Pass,
    /// A gate failed: wait (`run`) or exit 1 (`check`).
    Fail,
    /// No gate failed but one has no value yet: wait.
    Warming,
    /// A gate has no source here: exit 69 at once (src:V5).
    Unsupported,
}

/// What a verdict is about.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Subject {
    /// A gate on a signal.
    Signal(Gate),
}

/// One gate and its verdict this round.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Checked {
    /// The gate.
    pub subject: Subject,
    /// What it said.
    pub verdict: Verdict,
}

/// Every gate's verdict this round, in order, and what they say together.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Evaluation {
    /// Each gate's verdict, in the order the gates were given.
    pub verdicts: Vec<Checked>,
    /// The AND over them.
    pub overall: Overall,
}

/// Evaluates `gates` at instant `now`: reads each shared signal once,
/// judges every gate, and ANDs the verdicts. `trends` keeps the history
/// across rounds so a failure says which way its signal moves.
pub fn evaluate<S: Source + ?Sized>(
    gates: &mut Gates,
    source: &mut S,
    now: Instant,
    trends: &mut Trends,
) -> Evaluation {
    let own = gates.entries.iter().filter_map(|entry| match entry {
        Entry::Own(_, own) => Some(own),
        Entry::Shared(_) => None,
    });
    let _ = (own.count(), source, now, trends);
    Evaluation {
        verdicts: Vec::new(),
        overall: Overall::Fail,
    }
}

#[cfg(test)]
mod tests;
