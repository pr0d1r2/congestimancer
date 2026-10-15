use std::io;
use std::net::{SocketAddr, TcpListener};
use std::time::Duration;

use super::{connect_any, tcp_connect_time, with_port};

#[test]
fn a_bare_host_gets_the_default_port() {
    assert_eq!(with_port("example.com", 443), "example.com:443");
    assert_eq!(with_port("10.0.0.1", 22), "10.0.0.1:22");
    assert_eq!(with_port("::1", 22), "[::1]:22");
}

#[test]
fn a_target_with_a_port_keeps_it() {
    assert_eq!(with_port("example.com:8080", 443), "example.com:8080");
    assert_eq!(with_port("[::1]:8080", 443), "[::1]:8080");
    assert_eq!(with_port("10.0.0.1:22", 443), "10.0.0.1:22");
}

#[test]
fn a_listening_port_connects_within_the_timeout() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let took = tcp_connect_time("127.0.0.1", port, Duration::from_secs(5))?;
    assert!(took < Duration::from_secs(5));
    Ok(())
}

#[test]
fn a_closed_port_is_refused() -> io::Result<()> {
    let port = TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
    let target = format!("127.0.0.1:{port}");
    let err = tcp_connect_time(&target, 1, Duration::from_secs(5)).err();
    assert_eq!(
        err.map(|e| e.kind()),
        Some(io::ErrorKind::ConnectionRefused)
    );
    Ok(())
}

#[test]
fn no_time_left_is_a_timeout() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 9));
    let err = connect_any([addr].into_iter(), Duration::ZERO).err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::TimedOut));
}

#[test]
fn no_address_is_a_timeout() {
    let err = connect_any(std::iter::empty(), Duration::from_secs(1)).err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::TimedOut));
}
