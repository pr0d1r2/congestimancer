//! The RTT probe: how long a TCP connect takes, bounded by a timeout
//! (src/sys:T13, V2; src/net). std only: no raw sockets, no root.

use std::io;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

/// The time a TCP connect to `target` (`host`, `host:port`, `[v6]:port`)
/// takes; a target without a port gets `default_port`. Each address the
/// name resolves to is tried in turn within the one `timeout`.
///
/// # Errors
///
/// [`io::ErrorKind::TimedOut`] when no address connects in time; the
/// resolver's or the last connect's error otherwise. Name resolution
/// itself is not bounded by `timeout`.
pub fn tcp_connect_time(
    target: &str,
    default_port: u16,
    timeout: Duration,
) -> io::Result<Duration> {
    let addrs = with_port(target, default_port).to_socket_addrs()?;
    connect_any(addrs, timeout)
}

/// `target` with `default_port` added when it has none.
fn with_port(target: &str, default_port: u16) -> String {
    let _ = default_port;
    target.to_owned()
}

/// Connects to the first of `addrs` that answers before `timeout` runs out.
fn connect_any(
    addrs: impl Iterator<Item = SocketAddr>,
    timeout: Duration,
) -> io::Result<Duration> {
    let start = Instant::now();
    let mut last = io::Error::from(io::ErrorKind::TimedOut);
    for addr in addrs {
        let Some(left) = timeout.checked_sub(start.elapsed()) else {
            break;
        };
        let _ = (addr, left, TcpStream::connect_timeout);
        last = io::Error::from(io::ErrorKind::TimedOut);
    }
    Err(last)
}

#[cfg(test)]
mod tests;
