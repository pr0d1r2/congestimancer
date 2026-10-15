//! [`RemoteError`]: why a `--host` run cannot go on at all.

use std::fmt;

use crate::report::SCHEMA_VERSION;

/// Why a `--host` run stops, with the exit code it stops with.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RemoteError {
    /// `--host H` is not a host ssh may take: exit 64.
    Host {
        /// The host as given.
        host: String,
    },
    /// The remote cgmr speaks another JSON schema: exit 69
    /// (src/remote:V2).
    Schema {
        /// The `schema_version` it sent, as written; none when missing.
        found: Option<String>,
    },
    /// The remote cgmr's reply is not the schema's JSON: exit 69.
    Reply {
        /// What is wrong with it.
        reason: String,
    },
    /// `ssh` itself cannot be started here: exit 69.
    Spawn {
        /// Why, from the OS.
        reason: String,
    },
}

impl RemoteError {
    /// The exit code: 64 for usage, 69 for a host that cannot evaluate
    /// the gates.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Host { .. } => 64,
            Self::Schema { .. } | Self::Reply { .. } | Self::Spawn { .. } => 69,
        }
    }
}

impl fmt::Display for RemoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Host { host } => {
                write!(f, "--host {host:?}: not a host name for ssh")
            }
            Self::Schema { found } => schema(f, found.as_deref()),
            Self::Reply { reason } => {
                write!(f, "remote cgmr reply unreadable: {reason}")
            }
            Self::Spawn { reason } => write!(f, "cannot start ssh: {reason}"),
        }
    }
}

/// The schema mismatch, both versions named.
fn schema(f: &mut fmt::Formatter<'_>, found: Option<&str>) -> fmt::Result {
    let found = found.unwrap_or("none");
    write!(
        f,
        "remote cgmr speaks JSON schema {found}, \
         this cgmr reads schema {SCHEMA_VERSION}"
    )
}

impl std::error::Error for RemoteError {}

#[cfg(test)]
mod tests;
