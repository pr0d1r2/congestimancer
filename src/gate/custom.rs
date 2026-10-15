//! [`Custom`]: gates the user writes, `--when CMD` and `--when-file P`
//! (src/gate:T3, T4, V4, V5).

use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::{Threshold, Verdict};
use crate::signal::{Measure, Reason};

/// The shell `--when` runs its command with.
const SHELL: &str = "sh";

/// A gate the user writes instead of a signal threshold.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Custom {
    /// `--when CMD`: passes while `sh -c CMD` exits 0. CMD's output goes
    /// to cgmr's stderr, never its stdout (src/gate:V4).
    When(String),
    /// `--when-file P`: passes while P exists (src/gate:V5).
    WhenFile(PathBuf),
}

impl Custom {
    /// Runs the gate now.
    #[must_use]
    pub fn check(&self) -> Verdict {
        match self {
            Self::When(cmd) => run(SHELL, cmd),
            Self::WhenFile(path) => held(path.exists()),
        }
    }
}

/// Runs `shell -c cmd` with no stdin and both its outputs on cgmr's
/// stderr, so cgmr's stdout stays the gated command's (src/gate:V4).
fn run(shell: &str, cmd: &str) -> Verdict {
    let status = Command::new(shell)
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::null())
        .stdout(io::stderr())
        .stderr(io::stderr())
        .status();
    match status {
        Ok(status) => held(status.success()),
        Err(err) => Verdict::Unsupported {
            reason: Reason::from(format!("--when cannot start {shell}: {err}")),
        },
    }
}

/// The verdict on a custom gate that held or did not: a failure reads as
/// `no` against `yes`.
fn held(ok: bool) -> Verdict {
    if ok {
        Verdict::Pass
    } else {
        Verdict::Fail {
            value: Measure::Flag(false),
            threshold: Threshold::Value(Measure::Flag(true)),
            trend: None,
        }
    }
}

/// Renders as the switch: `--when CMD`, `--when-file P`.
impl fmt::Display for Custom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::When(cmd) => write!(f, "--when {cmd}"),
            Self::WhenFile(path) => write!(f, "--when-file {}", path.display()),
        }
    }
}

#[cfg(test)]
mod tests;
