use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::time::Duration;

use super::{RTT_PORT, RttSignal};
use crate::net::NetError;
use crate::signal::{Measure, Sample, Signal, SignalId};

/// Long enough for a loopback connect on a busy machine.
const TIMEOUT: Duration = Duration::from_secs(5);

/// A loopback listener's `127.0.0.1:PORT`, and the listener to keep.
fn listening() -> Option<(String, TcpListener)> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).ok()?;
    let port = listener.local_addr().ok()?.port();
    Some((format!("127.0.0.1:{port}"), listener))
}

/// A listener and a live connection to it.
type Taken = (TcpListener, TcpStream);

/// A loopback `127.0.0.1:PORT` nothing listens on, and the sockets that
/// keep `PORT` taken: the local end of a live connection, which no other
/// test's `bind(:0)` can be given, as a dropped listener's port can
/// (src/net:B1).
fn closed() -> Option<(String, Taken)> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).ok()?;
    let client = TcpStream::connect(listener.local_addr().ok()?).ok()?;
    let port = client.local_addr().ok()?.port();
    Some((format!("127.0.0.1:{port}"), (listener, client)))
}

/// The duration of a gauge sample, if it is one.
const fn duration(sample: &Sample) -> Option<Duration> {
    match sample {
        Sample::Gauge(Measure::Duration(d)) => Some(*d),
        _ => None,
    }
}

#[test]
fn rtt_is_the_connect_time_to_the_host() {
    let probed = listening().and_then(|(host, _listener)| {
        let mut rtt = RttSignal::new(Some(&host), TIMEOUT).ok()?;
        duration(&rtt.sample(SignalId::Rtt))
    });
    assert!(probed.is_some_and(|d| d < TIMEOUT), "{probed:?}");
}

#[test]
fn a_failed_connect_reads_the_timeout_so_the_gate_fails() {
    let probed = closed().and_then(|(host, _taken)| {
        let mut rtt = RttSignal::new(Some(&host), TIMEOUT).ok()?;
        duration(&rtt.sample(SignalId::Rtt))
    });
    assert_eq!(probed, Some(TIMEOUT));
}

#[test]
fn rtt_without_a_host_is_a_usage_error() {
    assert_eq!(
        RttSignal::new(None, TIMEOUT).err(),
        Some(NetError::RttWithoutHost)
    );
    assert_eq!(
        RttSignal::new(Some(" "), TIMEOUT).err(),
        Some(NetError::RttWithoutHost)
    );
}

#[test]
fn the_default_port_is_https() {
    assert_eq!(RTT_PORT, 443);
}

#[test]
fn other_signals_are_unsupported() {
    let rtt = RttSignal::new(Some("127.0.0.1"), TIMEOUT);
    let sample = rtt.ok().map(|mut rtt| rtt.sample(SignalId::Net));
    assert!(matches!(sample, Some(Sample::Unsupported(_))), "{sample:?}");
}
