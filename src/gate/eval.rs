//! [`evaluate`]: every gate in one round, AND over the verdicts
//! (src/gate:T2, T5, V2, V3, V6, V7).

use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

use super::{Custom, Gate, Verdict};
use crate::signal::{Reading, SignalId, SignalSet, Source, Trends, refresh};

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
    /// Reads no signal.
    Custom(Custom),
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

    /// Adds a `--when` or `--when-file` gate.
    pub fn push_custom(&mut self, custom: Custom) -> &mut Self {
        self.entries.push(Entry::Custom(custom));
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
        let mut list = f.debug_list();
        for entry in &self.entries {
            match entry {
                Entry::Shared(gate) | Entry::Own(gate, _) => list.entry(gate),
                Entry::Custom(custom) => list.entry(custom),
            };
        }
        list.finish()
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
    /// A `--when` or `--when-file` gate.
    Custom(Custom),
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
/// judges every gate, and joins the verdicts by AND. `trends` keeps the history
/// across rounds so a failure says which way its signal moves.
pub fn evaluate<S: Source + ?Sized>(
    gates: &mut Gates,
    source: &mut S,
    now: Instant,
    trends: &mut Trends,
) -> Evaluation {
    let wanted = gates.shared_signals();
    let readings: HashMap<SignalId, Reading> =
        refresh(source, wanted, now).into_iter().collect();
    for (id, reading) in &readings {
        trends.record(*id, reading);
    }
    let verdicts: Vec<Checked> = gates
        .entries
        .iter_mut()
        .map(|entry| entry.check(&readings, trends, now))
        .collect();
    let overall = Overall::of(verdicts.iter().map(|c| &c.verdict));
    Evaluation { verdicts, overall }
}

impl Gates {
    /// The signals the shared-source gates read, each once (src/gate:V7).
    fn shared_signals(&self) -> SignalSet {
        self.entries
            .iter()
            .filter_map(|entry| match entry {
                Entry::Shared(gate) => Some(gate.signal),
                Entry::Own(..) | Entry::Custom(_) => None,
            })
            .collect()
    }
}

impl Entry {
    /// This gate's verdict: shared gates judge the round's readings, an
    /// own-source gate reads its source now.
    fn check(
        &mut self,
        readings: &HashMap<SignalId, Reading>,
        trends: &Trends,
        now: Instant,
    ) -> Checked {
        match self {
            Self::Shared(gate) => {
                let reading = readings.get(&gate.signal);
                let reading = reading.unwrap_or(&Reading::Warming);
                let trend = trends.trend(gate.signal);
                Checked::signal(*gate, gate.judge(reading, trend))
            }
            Self::Own(gate, own) => {
                let reading = own.read(gate.signal, now);
                Checked::signal(*gate, gate.judge(&reading, None))
            }
            Self::Custom(custom) => Checked::custom(custom),
        }
    }
}

impl Checked {
    /// A custom gate's verdict, run now.
    fn custom(custom: &Custom) -> Self {
        Self {
            verdict: custom.check(),
            subject: Subject::Custom(custom.clone()),
        }
    }

    /// A signal gate's verdict.
    const fn signal(gate: Gate, verdict: Verdict) -> Self {
        Self {
            subject: Subject::Signal(gate),
            verdict,
        }
    }
}

impl Overall {
    /// The AND over `verdicts`: unsupported beats a failure, which beats
    /// warming; only all passes (or none at all) pass (src/gate:V2,
    /// V3, V6).
    fn of<'a>(verdicts: impl IntoIterator<Item = &'a Verdict>) -> Self {
        verdicts
            .into_iter()
            .map(|verdict| match verdict {
                Verdict::Pass => Self::Pass,
                Verdict::Warming => Self::Warming,
                Verdict::Fail { .. } => Self::Fail,
                Verdict::Unsupported { .. } => Self::Unsupported,
            })
            .max_by_key(|overall| overall.rank())
            .unwrap_or(Self::Pass)
    }

    /// How strongly this outcome overrides the others.
    const fn rank(self) -> u8 {
        match self {
            Self::Pass => 0,
            Self::Warming => 1,
            Self::Fail => 2,
            Self::Unsupported => 3,
        }
    }

    /// Whether the gates let the command run.
    #[must_use]
    pub const fn is_pass(self) -> bool {
        matches!(self, Self::Pass)
    }
}

#[cfg(test)]
mod tests;
