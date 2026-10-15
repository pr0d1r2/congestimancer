//! [`PowerError`]: why a thermal or power file did not parse.

use std::fmt;

/// Why sysfs or power-profiles-daemon text did not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PowerError {
    /// The text is not in the form its file takes.
    Malformed {
        /// The file, such as `temp` or `cur_state`.
        file: &'static str,
        /// The text as read, trimmed.
        text: String,
    },
}

impl PowerError {
    /// A [`PowerError::Malformed`] for `text` of `file`.
    pub(crate) fn malformed(file: &'static str, text: &str) -> Self {
        Self::Malformed {
            file,
            text: text.trim().to_owned(),
        }
    }
}

impl fmt::Display for PowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { file, text } => {
                write!(f, "malformed {file}: `{text}`")
            }
        }
    }
}

impl std::error::Error for PowerError {}

#[cfg(test)]
mod tests;
