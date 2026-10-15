//! [`CpuError`]: why a CPU file or setting did not parse.

use std::fmt;

/// Why `/proc` CPU text, or a `--load-window`, did not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CpuError {
    /// A line is not in the form its file takes.
    Malformed {
        /// The file, such as `/proc/loadavg`.
        file: &'static str,
        /// The line as read, trimmed.
        line: String,
    },
    /// A field the gate needs is not in the file.
    Missing {
        /// The file, such as `/proc/stat`.
        file: &'static str,
        /// The field, such as `procs_running`.
        key: &'static str,
    },
    /// A `--load-window` other than `1m`, `5m` or `15m`, as given.
    Window(String),
}

impl CpuError {
    /// A [`CpuError::Malformed`] for `line` of `file`.
    pub(crate) fn malformed(file: &'static str, line: &str) -> Self {
        Self::Malformed {
            file,
            line: line.trim().to_owned(),
        }
    }
}

impl fmt::Display for CpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { file, line } => {
                write!(f, "malformed {file} line: `{line}`")
            }
            Self::Missing { file, key } => write!(f, "{file} has no `{key}`"),
            Self::Window(input) => {
                write!(f, "--load-window takes 1m, 5m or 15m, not `{input}`")
            }
        }
    }
}

impl std::error::Error for CpuError {}

#[cfg(test)]
mod tests;
