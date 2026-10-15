//! [`BusError`]: why talking to logind over the system bus failed.

use std::{fmt, io};

/// Why the system bus gave no idle answer; it becomes the reason
/// `--idle` is unsupported (src:V5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum BusError {
    /// The socket failed.
    Io(String),
    /// The bus refused our credentials: its line as sent.
    Auth(String),
    /// Bytes that are not a D-Bus message of the shape expected.
    Malformed(&'static str),
    /// The bus or logind answered with an error.
    Remote {
        /// The D-Bus error name.
        name: String,
        /// Its message, empty when there is none.
        message: String,
    },
}

impl From<io::Error> for BusError {
    fn from(err: io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl fmt::Display for BusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "D-Bus: {err}"),
            Self::Auth(line) => write!(f, "D-Bus refused EXTERNAL: `{line}`"),
            Self::Malformed(what) => write!(f, "D-Bus: {what}"),
            Self::Remote { name, message } => {
                write!(f, "D-Bus: {name}: {message}")
            }
        }
    }
}

#[cfg(test)]
mod tests;
