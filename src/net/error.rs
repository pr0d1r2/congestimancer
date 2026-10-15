//! [`NetError`]: why a net switch cannot be gated on.

use std::fmt;

use crate::units::{Name, UnitError};

/// Why a net switch cannot be gated on: a usage error, exit 64.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NetError {
    /// `--net-iface I` names no interface on this machine (src/net:V1).
    UnknownIface {
        /// The interface as given.
        name: Name,
    },
    /// `--net-iface I` where `I` is not a name (src/net:V6).
    BadIface {
        /// Why, naming the input and the form expected.
        error: UnitError,
    },
    /// `--rtt` without `--rtt-host` (src/net:V4).
    RttWithoutHost,
}

impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownIface { name } => {
                write!(f, "--net-iface {name}: no such interface")
            }
            Self::BadIface { error } => write!(f, "--net-iface: {error}"),
            Self::RttWithoutHost => {
                f.write_str("--rtt needs --rtt-host H[:PORT]")
            }
        }
    }
}

impl std::error::Error for NetError {}

#[cfg(test)]
mod tests;
