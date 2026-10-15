use super::Bus;
use crate::presence::error::BusError;
use crate::presence::fixture::{Duplex, hex};
use crate::presence::wire::Call;

/// What the client says before its first message, as uid 1000.
const AUTH: &[u8] = b"\0AUTH EXTERNAL 31303030\r\nBEGIN\r\n";

/// logind's `ListSessions`.
const LIST: Call<'static> = Call {
    path: "/org/freedesktop/login1",
    interface: "org.freedesktop.login1.Manager",
    member: "ListSessions",
    destination: "org.freedesktop.login1",
    args: &[],
};

#[test]
fn open_authenticates_then_says_hello() {
    let mut duplex = Duplex::serving("stream-headless");
    assert!(Bus::open(&mut duplex, 1000).is_ok());
    let mut want = AUTH.to_vec();
    want.extend(hex("wire/hello-call"));
    assert_eq!(duplex.output, want);
}

#[test]
fn a_refused_login_is_an_auth_error() {
    let mut duplex = Duplex::serving("stream-rejected");
    let err = Bus::open(&mut duplex, 1000).err();
    assert_eq!(err, Some(BusError::Auth("REJECTED EXTERNAL".to_owned())));
}

#[test]
fn a_silent_bus_is_an_io_error() {
    let mut duplex = Duplex::default();
    let err = Bus::open(&mut duplex, 0).err();
    assert!(matches!(err, Some(BusError::Io(_))), "{err:?}");
}

#[test]
fn a_call_skips_signals_to_reach_its_reply() {
    let mut duplex = Duplex::serving("stream-desktop");
    let reply = Bus::open(&mut duplex, 1000)
        .and_then(|mut bus| bus.call(&LIST))
        .and_then(|reply| reply.session_paths());
    assert_eq!(reply.map(|paths| paths.len()), Ok(3));
    assert!(duplex.output.ends_with(&hex("wire/list-sessions-call")));
}

#[test]
fn a_call_after_the_stream_ends_is_an_io_error() {
    let mut duplex = Duplex::serving("stream-headless");
    let mut bus = Bus::open(&mut duplex, 1000);
    let _ = bus.as_mut().map(|bus| bus.call(&LIST));
    let err = bus.and_then(|mut bus| bus.call(&LIST)).err();
    assert!(matches!(err, Some(BusError::Io(_))), "{err:?}");
}
