//! The RTT probe: how long a TCP connect takes, bounded by a timeout
//! (src/sys:T13, V2; src/net). std only: no raw sockets, no root.

use std::io;
use std::net::{Ipv6Addr, SocketAddr, TcpStream, ToSocketAddrs};
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
    if target.parse::<SocketAddr>().is_ok() {
        return target.to_owned();
    }
    if target.parse::<Ipv6Addr>().is_ok() {
        return format!("[{target}]:{default_port}");
    }
    match target.rsplit_once(':') {
        Some((_, port)) if port.parse::<u16>().is_ok() => target.to_owned(),
        _ => format!("{target}:{default_port}"),
    }
}

/// Connects to the first of `addrs` that answers before `timeout` runs out.
fn connect_any(
    addrs: impl Iterator<Item = SocketAddr>,
    timeout: Duration,
) -> io::Result<Duration> {
    let start = Instant::now();
    let mut last = io::Error::from(io::ErrorKind::TimedOut);
    for addr in addrs {
        let left = timeout.saturating_sub(start.elapsed());
        if left.is_zero() {
            break;
        }
        match TcpStream::connect_timeout(&addr, left) {
            Ok(_) => return Ok(start.elapsed()),
            Err(err) => last = err,
        }
    }
    Err(last)
}

#[cfg(test)]
mod tests;
