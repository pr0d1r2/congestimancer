//! [`parse_reply`]: what the remote `cgmr check --json` said, read back
//! from `src/report`'s JSON schema (src/remote:T2, V1, V2).

use std::fmt;

use super::RemoteError;
use crate::gate::Overall;

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
    pub const fn overall(&self) -> Overall {
        Overall::Pass
    }

    /// One line per gate that did not pass, for `--verbose` and the
    /// heartbeat: `--load 2: fail on box`.
    #[must_use]
    pub const fn block_reasons(&self) -> Vec<String> {
        Vec::new()
    }
}

impl fmt::Display for RemoteVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("")
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
    let _ = text;
    Err(RemoteError::Reply {
        reason: String::new(),
    })
}

#[cfg(test)]
mod tests;
