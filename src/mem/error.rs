//! [`MemError`]: why a memory file did not parse.

use std::fmt;

/// Why `/proc` memory text did not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MemError {
    /// A line is not in the form its file takes.
    Malformed {
        /// The file, such as `/proc/vmstat`.
        file: &'static str,
        /// The line as read, trimmed.
        line: String,
    },
    /// A field the gate needs is not in the file.
    Missing {
        /// The file, such as `/proc/vmstat`.
        file: &'static str,
        /// The field, such as `pswpin`.
        key: &'static str,
    },
}

impl MemError {
    /// A [`MemError::Malformed`] for `line` of `file`.
    pub(crate) fn malformed(file: &'static str, line: &str) -> Self {
        Self::Malformed {
            file,
            line: line.trim().to_owned(),
        }
    }
}

impl fmt::Display for MemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { file, line } => {
                write!(f, "malformed {file} line: `{line}`")
            }
            Self::Missing { file, key } => write!(f, "{file} has no `{key}`"),
        }
    }
}

impl std::error::Error for MemError {}

#[cfg(test)]
mod tests;
