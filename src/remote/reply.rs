//! [`parse_reply`]: what the remote `cgmr check --json` said, read back
//! from `src/report`'s JSON schema (src/remote:T2, V1, V2).

use std::fmt;

use super::RemoteError;
use super::json::{self, Value};
use crate::gate::Overall;
use crate::report::SCHEMA_VERSION;

/// One gate's verdict on the remote host, as its one word says it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RemoteVerdict {
    /// `pass`.
    Pass,
    /// `fail`.
    Fail,
    /// `warming`.
    Warming,
    /// `unsupported`: exit 69.
    Unsupported,
}

/// One gate the remote host judged.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct RemoteGate {
    /// The switch, without `--`: `load`, `when`.
    pub switch: String,
    /// The threshold as the switch takes it: `2`, `1GiB`.
    pub threshold: String,
    /// What the remote host said.
    pub verdict: RemoteVerdict,
}

/// One signal the remote host read.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct RemoteSignal {
    /// The signal's name: `load`.
    pub id: String,
    /// Its value as the JSON wrote it (`1.5`, `true`, `high`); none when
    /// it has no value.
    pub value: Option<String>,
    /// Its unit (`number`, `bytes`); none when it has no value.
    pub unit: Option<String>,
    /// Which way it moves, once known.
    pub trend: Option<String>,
    /// Whether the remote host has a source for it.
    pub supported: bool,
    /// Where the value comes from: `host` or `cgroup`.
    pub source: String,
}

/// Everything one remote `check --json` reported.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct RemoteReport {
    /// The remote host's own name for itself.
    pub host: String,
    /// The signals it read, in order.
    pub signals: Vec<RemoteSignal>,
    /// The gates it judged, in order.
    pub gates: Vec<RemoteGate>,
}

impl RemoteReport {
    /// The AND over the gates, as `src/gate` joins them: unsupported
    /// beats a failure, which beats warming; no gates pass.
    #[must_use]
    pub fn overall(&self) -> Overall {
        let worst = self.gates.iter().map(|g| g.verdict).max_by_key(rank);
        worst.map_or(Overall::Pass, RemoteVerdict::overall)
    }

    /// One line per gate that did not pass, for `--verbose` and the
    /// heartbeat: `--load 2: fail on box`.
    #[must_use]
    pub fn block_reasons(&self) -> Vec<String> {
        let blocking = self.gates.iter().filter(|g| g.verdict.blocks());
        let line = |g: &RemoteGate| {
            let (switch, threshold, host) =
                (&g.switch, &g.threshold, &self.host);
            format!("--{switch} {threshold}: {} on {host}", g.verdict)
        };
        blocking.map(line).collect()
    }
}

impl RemoteVerdict {
    /// The verdict a JSON `verdict` word names.
    fn from_word(word: &str) -> Option<Self> {
        match word {
            "pass" => Some(Self::Pass),
            "fail" => Some(Self::Fail),
            "warming" => Some(Self::Warming),
            "unsupported" => Some(Self::Unsupported),
            _ => None,
        }
    }

    /// Whether it keeps the command waiting.
    const fn blocks(self) -> bool {
        !matches!(self, Self::Pass)
    }

    /// The same outcome as the local gate's.
    const fn overall(self) -> Overall {
        match self {
            Self::Pass => Overall::Pass,
            Self::Fail => Overall::Fail,
            Self::Warming => Overall::Warming,
            Self::Unsupported => Overall::Unsupported,
        }
    }
}

/// How strongly a verdict overrides the others (src/gate:V2, V3, V6).
const fn rank(verdict: &RemoteVerdict) -> u8 {
    match verdict {
        RemoteVerdict::Pass => 0,
        RemoteVerdict::Warming => 1,
        RemoteVerdict::Fail => 2,
        RemoteVerdict::Unsupported => 3,
    }
}

impl fmt::Display for RemoteVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Warming => "warming",
            Self::Unsupported => "unsupported",
        })
    }
}

/// Reads the remote `cgmr check --json` line.
///
/// # Errors
///
/// [`RemoteError::Schema`] when its `schema_version` is not the one this
/// cgmr speaks (src/remote:V2); [`RemoteError::Reply`] when it is not the
/// schema's JSON.
pub fn parse_reply(text: &str) -> Result<RemoteReport, RemoteError> {
    let garbled = |reason: String| RemoteError::Reply { reason };
    let value = json::parse(text).map_err(garbled)?;
    check_version(&value)?;
    report(&value)
        .ok_or_else(|| garbled("a field is missing or mistyped".into()))
}

/// Refuses any `schema_version` but this cgmr's (src/remote:V2).
fn check_version(value: &Value) -> Result<(), RemoteError> {
    let found = match value.get("schema_version") {
        Some(Value::Number(n)) if n.parse() == Ok(SCHEMA_VERSION) => {
            return Ok(());
        }
        Some(Value::Number(n)) => Some(n.clone()),
        Some(_) => Some("of the wrong type".to_owned()),
        None => None,
    };
    Err(RemoteError::Schema { found })
}

/// The report a schema v1 object holds.
fn report(value: &Value) -> Option<RemoteReport> {
    Some(RemoteReport {
        host: text(value, "host")?,
        signals: each(value, "signals", signal)?,
        gates: each(value, "gates", gate)?,
    })
}

/// Each item of the array `key`, read with `read`; none if any fails.
fn each<T>(
    value: &Value,
    key: &str,
    read: fn(&Value) -> Option<T>,
) -> Option<Vec<T>> {
    value.get(key)?.as_list()?.iter().map(read).collect()
}

/// One `signals` entry.
fn signal(value: &Value) -> Option<RemoteSignal> {
    Some(RemoteSignal {
        id: text(value, "id")?,
        value: scalar(value.get("value")?)?,
        unit: maybe_text(value, "unit")?,
        trend: maybe_text(value, "trend")?,
        supported: match value.get("supported")? {
            Value::Bool(on) => *on,
            _ => return None,
        },
        source: text(value, "source")?,
    })
}

/// One `gates` entry.
fn gate(value: &Value) -> Option<RemoteGate> {
    Some(RemoteGate {
        switch: text(value, "switch")?,
        threshold: text(value, "threshold")?,
        verdict: RemoteVerdict::from_word(value.get("verdict")?.as_text()?)?,
    })
}

/// The string member `key`.
fn text(value: &Value, key: &str) -> Option<String> {
    value.get(key)?.as_text().map(str::to_owned)
}

/// The member `key`, a string or `null`; none (outer) when it is neither.
fn maybe_text(value: &Value, key: &str) -> Option<Option<String>> {
    match value.get(key)? {
        Value::Null => Some(None),
        Value::Text(text) => Some(Some(text.clone())),
        _ => None,
    }
}

/// A signal's value as text: a number as written, a flag, a string, or
/// `null`; none (outer) for an array or object.
fn scalar(value: &Value) -> Option<Option<String>> {
    match value {
        Value::Null => Some(None),
        Value::Bool(on) => Some(Some(on.to_string())),
        Value::Number(text) | Value::Text(text) => Some(Some(text.clone())),
        Value::List(_) | Value::Object(_) => None,
    }
}

#[cfg(test)]
mod tests;
