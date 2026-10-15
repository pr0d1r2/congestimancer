//! Linux interface counters: `/proc/net/dev` for the totals,
//! `/sys/class/net/<iface>/flags` for up and loopback (src/net:T1).

use std::fmt;
use std::io;

use crate::signal::Root;
use crate::sys::{InterfaceCounters, LinkCounters, read_text};

/// Counters per `/proc/net/dev` line: 8 receive, then 8 transmit.
const FIELDS: usize = 16;
/// Where the transmit counters start.
const TX: usize = 8;
/// `IFF_UP`, `IFF_LOOPBACK` (`linux/if.h`).
const IFF_UP: u32 = 0x1;
const IFF_LOOPBACK: u32 = 0x8;

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
    text: &str,
) -> Result<Vec<InterfaceCounters>, NetDevError> {
    text.lines()
        .enumerate()
        .skip(2)
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(at, line)| {
            interface(line).ok_or_else(|| NetDevError {
                line: at.saturating_add(1),
                text: line.to_owned(),
            })
        })
        .collect()
}

/// One `name: counters` line, up and loopback left `false`.
fn interface(line: &str) -> Option<InterfaceCounters> {
    let (name, counters) = line.split_once(':')?;
    let values = counters
        .split_whitespace()
        .map(|n| n.parse::<u64>().ok())
        .collect::<Option<Vec<u64>>>()
        .filter(|values| values.len() == FIELDS)?;
    Some(InterfaceCounters {
        name: name.trim().to_owned(),
        up: false,
        loopback: false,
        rx: link(&values, 0)?,
        tx: link(&values, TX)?,
    })
}

/// Bytes, packets and errors: the first three counters from `from`.
fn link(values: &[u64], from: usize) -> Option<LinkCounters> {
    match values.get(from..)? {
        [bytes, packets, errors, ..] => Some(LinkCounters {
            bytes: *bytes,
            packets: *packets,
            errors: *errors,
        }),
        _ => None,
    }
}

/// The `IFF_*` bits of a `/sys/class/net/<iface>/flags` file (`0x1003`);
/// `None` when it is not `0x` hex.
#[must_use]
pub fn parse_if_flags(text: &str) -> Option<u32> {
    let hex = text.trim().strip_prefix("0x")?;
    u32::from_str_radix(hex, 16).ok()
}

/// Every interface under `root`, with up and loopback from its flags
/// file; an interface without one counts as up, and as loopback only when
/// named `lo`.
///
/// # Errors
///
/// `/proc/net/dev` unreadable (Unsupported when absent) or malformed
/// ([`io::ErrorKind::InvalidData`]).
pub fn linux_interfaces(root: &Root) -> io::Result<Vec<InterfaceCounters>> {
    let text = read_text(root, "/proc/net/dev")?;
    let mut found = parse_net_dev(&text)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    for iface in &mut found {
        flag(root, iface);
    }
    Ok(found)
}

/// Sets up and loopback of `iface` from its sysfs flags file.
fn flag(root: &Root, iface: &mut InterfaceCounters) {
    let path = format!("/sys/class/net/{}/flags", iface.name);
    let bits = read_text(root, &path).ok();
    if let Some(bits) = bits.as_deref().and_then(parse_if_flags) {
        iface.up = bits & IFF_UP != 0;
        iface.loopback = bits & IFF_LOOPBACK != 0;
    } else {
        iface.up = true;
        iface.loopback = iface.name == "lo";
    }
}

#[cfg(test)]
mod tests;
