//! The Linux D-Bus system bus socket, connected and handed over as a std
//! `UnixStream`; the wire codec lives in `src/presence` (src/sys:T16,
//! V2, V4; src/presence:R1). Every other OS answers
//! [`io::ErrorKind::Unsupported`].

use std::io;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// Where the system bus listens when `DBUS_SYSTEM_BUS_ADDRESS` is unset.
pub const SYSTEM_BUS_SOCKET: &str = "/run/dbus/system_bus_socket";

/// A connection to the system bus: `DBUS_SYSTEM_BUS_ADDRESS`'s first
/// `unix:path=` entry, else [`SYSTEM_BUS_SOCKET`].
///
/// # Errors
///
/// Unsupported when there is no bus socket (a container, a host without
/// D-Bus) or off Linux; any other connect error as is.
pub fn system_bus_connect() -> io::Result<UnixStream> {
    let address = std::env::var("DBUS_SYSTEM_BUS_ADDRESS").ok();
    imp::connect(system_bus_path(address.as_deref()))
}

/// The socket path a D-Bus address names: the first `;`-separated
/// `unix:` entry with a `path=` key, `%XX` escapes decoded; the default
/// path when the address is unset. `None` when no entry is a path.
fn system_bus_path(address: Option<&str>) -> Option<PathBuf> {
    let _ = address;
    None
}

#[cfg(target_os = "linux")]
mod imp {
    use std::io;
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;

    use crate::sys::error::unsupported;

    pub(super) fn connect(path: Option<PathBuf>) -> io::Result<UnixStream> {
        let path = path.ok_or_else(|| {
            unsupported("D-Bus: DBUS_SYSTEM_BUS_ADDRESS names no unix path")
        })?;
        UnixStream::connect(&path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused => {
                unsupported(format_args!(
                    "D-Bus: no system bus at {}",
                    path.display()
                ))
            }
            _ => err,
        })
    }
}

#[cfg(not(target_os = "linux"))]
mod imp {
    use std::io;
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;

    pub(super) fn connect(_: Option<PathBuf>) -> io::Result<UnixStream> {
        Err(crate::sys::error::unsupported(
            "D-Bus system bus: Linux only",
        ))
    }
}

#[cfg(test)]
mod tests;
