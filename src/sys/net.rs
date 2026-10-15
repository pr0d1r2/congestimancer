//! Network: macOS per-interface counters (src/sys:T12, V1, V3).
//!
//! Counters come from the `NET_RT_IFLIST2` sysctl, whose `if_msghdr2`
//! carries 64-bit `if_data64`; `getifaddrs` gives the 32-bit `if_data`,
//! which wraps every 4 GiB, so under load a rate would never warm up
//! (src/sys:R5). Linux reads `/proc/net/dev` through `read_text`.

use std::io;

use super::raw::{ne_i32, ne_u16, ne_u64};

/// One direction's cumulative counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct LinkCounters {
    /// Bytes.
    pub bytes: u64,
    /// Packets.
    pub packets: u64,
    /// Errors.
    pub errors: u64,
}

/// One network interface and its counters since boot.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct InterfaceCounters {
    /// The interface name, such as `en0`.
    pub name: String,
    /// `IFF_UP`.
    pub up: bool,
    /// `IFF_LOOPBACK`.
    pub loopback: bool,
    /// Received.
    pub rx: LinkCounters,
    /// Sent.
    pub tx: LinkCounters,
}

/// Every interface with its counters.
///
/// # Errors
///
/// The sysctl error; Unsupported off macOS.
pub fn interface_counters() -> io::Result<Vec<InterfaceCounters>> {
    imp::iflist2().map(|raw| decode_iflist2(&raw))
}

/// `if_msghdr2` field offsets (`net/if.h`, `#pragma pack(4)`), checked
/// against `libc` on macOS in the tests.
mod at {
    pub(super) const TYPE: usize = 3;
    pub(super) const ADDRS: usize = 4;
    pub(super) const FLAGS: usize = 8;
    /// `ifm_data`, an `if_data64`, and its counters.
    pub(super) const DATA: usize = 32;
    pub(super) const IPACKETS: usize = DATA + 24;
    pub(super) const IERRORS: usize = DATA + 32;
    pub(super) const OPACKETS: usize = DATA + 40;
    pub(super) const OERRORS: usize = DATA + 48;
    pub(super) const IBYTES: usize = DATA + 64;
    pub(super) const OBYTES: usize = DATA + 72;
    /// The `sockaddr_dl` after the header: name length, then the name.
    pub(super) const SDL: usize = 160;
    pub(super) const SDL_NLEN: usize = SDL + 5;
    pub(super) const SDL_DATA: usize = SDL + 8;
}

/// `RTM_IFINFO2`: an interface message.
const RTM_IFINFO2: u8 = 0x12;
/// `RTA_IFP`: the message carries the interface's `sockaddr_dl`.
const RTA_IFP: i32 = 0x10;
/// `IFF_UP`, `IFF_LOOPBACK`.
const IFF_UP: i32 = 0x1;
const IFF_LOOPBACK: i32 = 0x8;

/// Every routing message in a sysctl buffer: each starts with its own
/// `u16` length; a length too short to hold a header ends the walk.
fn messages(buf: &[u8]) -> Vec<&[u8]> {
    let mut found = Vec::new();
    let mut rest = buf;
    while let Some(len) = ne_u16(rest, 0).map(usize::from) {
        let Some((msg, tail)) = rest.split_at_checked(len) else {
            break;
        };
        if len <= at::TYPE {
            break;
        }
        found.push(msg);
        rest = tail;
    }
    found
}

/// The interfaces in a `NET_RT_IFLIST2` buffer; other messages skipped.
fn decode_iflist2(buf: &[u8]) -> Vec<InterfaceCounters> {
    messages(buf)
        .into_iter()
        .filter(|msg| msg.get(at::TYPE) == Some(&RTM_IFINFO2))
        .filter_map(decode_interface)
        .collect()
}

/// One `if_msghdr2` and its `sockaddr_dl` name; `None` without the
/// name (no `RTA_IFP`) or when short.
fn decode_interface(msg: &[u8]) -> Option<InterfaceCounters> {
    if ne_i32(msg, at::ADDRS)? & RTA_IFP == 0 {
        return None;
    }
    let flags = ne_i32(msg, at::FLAGS)?;
    Some(InterfaceCounters {
        name: name_of(msg)?,
        up: flags & IFF_UP != 0,
        loopback: flags & IFF_LOOPBACK != 0,
        rx: link(msg, at::IBYTES, at::IPACKETS, at::IERRORS)?,
        tx: link(msg, at::OBYTES, at::OPACKETS, at::OERRORS)?,
    })
}

/// The interface name in the `sockaddr_dl` after the header.
fn name_of(msg: &[u8]) -> Option<String> {
    let len = usize::from(*msg.get(at::SDL_NLEN)?);
    let name = msg.get(at::SDL_DATA..at::SDL_DATA.checked_add(len)?)?;
    Some(String::from_utf8_lossy(name).into_owned())
}

/// One direction's counters at the given offsets.
fn link(
    msg: &[u8],
    bytes: usize,
    packets: usize,
    errors: usize,
) -> Option<LinkCounters> {
    Some(LinkCounters {
        bytes: ne_u64(msg, bytes)?,
        packets: ne_u64(msg, packets)?,
        errors: ne_u64(msg, errors)?,
    })
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code, reason = "sysctl FFI (src/sys:C)")]
mod imp {
    use std::io;

    pub(super) fn iflist2() -> io::Result<Vec<u8>> {
        crate::sys::sysctl::sized(route)
    }

    /// One `sysctl` read of `{CTL_NET, PF_ROUTE, 0, 0, NET_RT_IFLIST2, 0}`.
    fn route(buf: *mut libc::c_void, len: &mut usize) -> io::Result<()> {
        let mut mib =
            [libc::CTL_NET, libc::PF_ROUTE, 0, 0, libc::NET_RT_IFLIST2, 0];
        // SAFETY: `mib` is 6 valid ints; `sized` passes a null `buf` (a
        // size query) or one of `*len` writable bytes, and `len` is a
        // valid `size_t` the kernel updates in place.
        let rc = unsafe {
            libc::sysctl(mib.as_mut_ptr(), 6, buf, len, std::ptr::null_mut(), 0)
        };
        if rc == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use std::io;

    pub(super) fn iflist2() -> io::Result<Vec<u8>> {
        Err(crate::sys::error::unsupported("NET_RT_IFLIST2: macOS only"))
    }
}

#[cfg(test)]
mod tests;
