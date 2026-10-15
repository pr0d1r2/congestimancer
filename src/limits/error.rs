//! [`LimitsError`]: why a limits file did not parse.

use std::fmt;

/// Why `/proc` limits text did not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LimitsError {
    /// The text is not in the form its file takes.
    Malformed {
        /// The file, such as `file-nr` or `pid_max`.
        file: &'static str,
        /// The text as read, trimmed.
        text: String,
    },
}

impl LimitsError {
    /// A [`LimitsError::Malformed`] for `text` of `file`.
    pub(crate) fn malformed(file: &'static str, text: &str) -> Self {
        Self::Malformed {
            file,
            text: text.trim().to_owned(),
        }
    }
}

impl fmt::Display for LimitsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { file, text } => {
                write!(f, "malformed {file}: `{text}`")
            }
        }
    }
}

impl std::error::Error for LimitsError {}

#[cfg(test)]
mod tests;
