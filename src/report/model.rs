//! [`StatusRow`] and [`StatusReport`]: what `status` and `check` render,
//! and [`switch_name`], the switch a gate came from (src/report:T1).

use std::borrow::Cow;

use crate::gate::{Adjust, Checked, Custom, Gate, Subject};
use crate::signal::{Reading, SignalId, Trend};
use crate::units::Bytes;

/// The source a row reads when the caller names none.
const HOST: &str = "host";

/// One signal as `status` shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct StatusRow {
    /// The signal.
    pub id: SignalId,
    /// What it reads now.
    pub reading: Reading,
    /// Which way it moves, once known.
    pub trend: Option<Trend>,
    /// Where the value comes from: `host`, or `cgroup` beside it.
    pub source: Cow<'static, str>,
}

impl StatusRow {
    /// `id` reading `reading` on the host, with no trend.
    #[must_use]
    pub const fn new(id: SignalId, reading: Reading) -> Self {
        Self {
            id,
            reading,
            trend: None,
            source: Cow::Borrowed(HOST),
        }
    }

    /// The same row, moving `trend`.
    #[must_use]
    pub const fn with_trend(mut self, trend: Option<Trend>) -> Self {
        self.trend = trend;
        self
    }

    /// The same row, read from `source` (such as `cgroup`).
    #[must_use]
    pub fn with_source(mut self, source: impl Into<Cow<'static, str>>) -> Self {
        self.source = source.into();
        self
    }
}

/// A memory peak `--needs-mem auto` learned, as `status` lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct LearnedPeak {
    /// The `--mem-key`.
    pub key: String,
    /// The peak kept for it, in bytes.
    pub bytes: Bytes,
}

impl LearnedPeak {
    /// The peak `bytes` kept for `key`.
    #[must_use]
    pub fn new(key: impl Into<String>, bytes: Bytes) -> Self {
        Self {
            key: key.into(),
            bytes,
        }
    }
}

/// Everything one `status` or `check` run reports.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct StatusReport {
    /// The machine read: the local host name, or `--host`'s.
    pub host: String,
    /// The signals read, in order.
    pub signals: Vec<StatusRow>,
    /// The gates judged, in order; empty for `status`.
    pub gates: Vec<Checked>,
    /// The memory peaks learned per key (`--needs-mem auto`); listed by
    /// `status` only when there are some.
    pub learned: Vec<LearnedPeak>,
}

impl StatusReport {
    /// A report on `host` with nothing in it yet.
    #[must_use]
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            ..Self::default()
        }
    }
}

/// The switch a gate came from, without `--`: `load`, `all-load`,
/// `needs-mem`, `when`.
#[must_use]
pub fn switch_name(subject: &Subject) -> String {
    match subject {
        Subject::Signal(gate) => signal_switch(gate),
        Subject::Custom(Custom::When(_)) => "when".to_owned(),
        Subject::Custom(Custom::WhenFile(_)) => "when-file".to_owned(),
    }
}

/// The switch of a signal gate: per-CPU signals read whole are the
/// `all-` switches, memory less reservations is `needs-mem`.
fn signal_switch(gate: &Gate) -> String {
    let name = gate.signal.name();
    match (gate.signal, gate.adjust) {
        (SignalId::Mem, Adjust::Unreserved(_)) => "needs-mem".to_owned(),
        (
            SignalId::Load | SignalId::RunQueue | SignalId::Mem,
            Adjust::Whole,
        ) => {
            format!("all-{name}")
        }
        _ => name.to_owned(),
    }
}

#[cfg(test)]
mod tests;
