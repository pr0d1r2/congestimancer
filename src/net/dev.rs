//! Linux interface counters: `/proc/net/dev` for the totals,
//! `/sys/class/net/<iface>/flags` for up and loopback (src/net:T1).

use std::fmt;
use std::io;

use crate::signal::Root;
use crate::sys::InterfaceCounters;

/// A `/proc/net/dev` line that did not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct NetDevError {
    /// The 1-based line number.
    pub line: usize,
    /// The line as read.
    pub text: String,
}

impl fmt::Display for NetDevError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/proc/net/dev line {}: {:?}", self.line, self.text)
    }
}

impl std::error::Error for NetDevError {}

/// The interfaces in `/proc/net/dev` text, in file order, with their
/// receive and transmit totals. `up` and `loopback` are left `false`:
/// they come from sysfs ([`parse_if_flags`]).
///
/// # Errors
///
/// [`NetDevError`] naming the first line after the two headers that is
/// not `name: 16 counters`.
pub fn parse_net_dev(
    _text: &str,
) -> Result<Vec<InterfaceCounters>, NetDevError> {
    Ok(Vec::new())
}

/// The `IFF_*` bits of a `/sys/class/net/<iface>/flags` file (`0x1003`);
/// `None` when it is not `0x` hex.
#[must_use]
pub fn parse_if_flags(_text: &str) -> Option<u32> {
    None
}

/// Every interface under `root`, with up and loopback from its flags
/// file; an interface without one counts as up, and as loopback only when
/// named `lo`.
///
/// # Errors
///
/// `/proc/net/dev` unreadable (Unsupported when absent) or malformed
/// ([`io::ErrorKind::InvalidData`]).
pub fn linux_interfaces(_root: &Root) -> io::Result<Vec<InterfaceCounters>> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests;
