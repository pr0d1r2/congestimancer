//! [`RemoteError`]: why a `--host` run cannot go on at all.

use std::fmt;

/// Why a `--host` run stops, with the exit code it stops with.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RemoteError {
    /// `--host H` is not a host ssh may take: exit 64.
    Host {
        /// The host as given.
        host: String,
    },
}

impl RemoteError {
    /// The exit code: 64 for usage.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Host { .. } => 64,
        }
    }
}

impl fmt::Display for RemoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Host { host } => {
                write!(f, "--host {host:?}: not a host name for ssh")
            }
        }
    }
}

impl std::error::Error for RemoteError {}

#[cfg(test)]
mod tests;
