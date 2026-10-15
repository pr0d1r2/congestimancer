use proptest::prelude::*;

use super::{BusError, Call, HEAD, Message, Value, decode, encode, frame_len};
use crate::presence::fixture::hex;

/// The logind manager's object path.
const LOGIN1: &str = "/org/freedesktop/login1";

/// The `Hello` call every connection starts with.
const HELLO: Call<'static> = Call {
    path: "/org/freedesktop/DBus",
    interface: "org.freedesktop.DBus",
    member: "Hello",
    destination: "org.freedesktop.DBus",
    args: &[],
};

/// The message in fixture `wire/<name>`.
fn message(name: &str) -> Result<Message, BusError> {
    decode(&hex(&format!("wire/{name}")))
}

#[test]
fn hello_encodes_as_the_spec_lays_it_out() {
    assert_eq!(encode(&HELLO, 1), Ok(hex("wire/hello-call")));
}

#[test]
fn list_sessions_encodes_with_no_body() {
    let call = Call {
        path: LOGIN1,
        interface: "org.freedesktop.login1.Manager",
        member: "ListSessions",
        destination: "org.freedesktop.login1",
        args: &[],
    };
    assert_eq!(encode(&call, 2), Ok(hex("wire/list-sessions-call")));
}

#[test]
fn a_property_get_carries_its_two_strings() {
    let call = Call {
        path: "/org/freedesktop/login1/session/_32",
        interface: "org.freedesktop.DBus.Properties",
        member: "Get",
        destination: "org.freedesktop.login1",
        args: &["org.freedesktop.login1.Session", "Class"],
    };
    assert_eq!(encode(&call, 3), Ok(hex("wire/get-class-call")));
}

#[test]
fn a_hello_reply_names_us() {
    let reply = message("hello-reply");
    let reply = reply.as_ref().map(|m| (m.kind, m.reply_serial));
    assert_eq!(reply, Ok((2, Some(1))));
    let name = message("hello-reply").and_then(|m| m.string());
    assert_eq!(name, Ok(":1.42".to_owned()));
}

#[test]
fn a_signal_answers_no_call() {
    let signal = message("name-acquired");
    let signal = signal.map(|m| (m.kind, m.reply_serial));
    assert_eq!(signal, Ok((4, None)));
}

#[test]
fn list_sessions_gives_every_object_path() {
    let paths = message("list-sessions-reply").and_then(|m| m.session_paths());
    let want = ["_32", "c1", "_37"].map(|id| format!("{LOGIN1}/session/{id}"));
    assert_eq!(paths, Ok(want.to_vec()));
    let none = message("list-sessions-empty").and_then(|m| m.session_paths());
    assert_eq!(none, Ok(Vec::new()));
}

#[test]
fn property_replies_give_their_values() {
    let get = |name| message(name).and_then(|m| m.variant());
    assert_eq!(get("get-class-reply"), Ok(Value::Str("user".to_owned())));
    assert_eq!(get("get-remote-reply"), Ok(Value::Bool(false)));
    assert_eq!(
        get("get-since-reply"),
        Ok(Value::U64(1_700_000_000_000_000))
    );
}

#[test]
fn an_error_reply_is_the_remote_error() {
    let err = message("get-error-reply").and_then(Message::reply);
    assert_eq!(
        err,
        Err(BusError::Remote {
            name: "org.freedesktop.DBus.Error.UnknownObject".to_owned(),
            message: "Unknown object".to_owned(),
        })
    );
    let ok = message("hello-reply").and_then(Message::reply);
    assert_eq!(ok, message("hello-reply"));
}

#[test]
fn the_head_tells_the_whole_length() {
    for name in ["hello-reply", "list-sessions-reply", "get-since-reply"] {
        let bytes = hex(&format!("wire/{name}"));
        let head = bytes.get(..HEAD).unwrap_or_default();
        assert_eq!(frame_len(head), Ok(bytes.len()), "{name}");
    }
}

#[test]
fn a_body_of_another_signature_is_malformed() {
    let reply = message("hello-reply");
    assert!(matches!(
        reply.and_then(|m| m.variant()),
        Err(BusError::Malformed(_))
    ));
    let paths = message("get-class-reply").and_then(|m| m.session_paths());
    assert!(matches!(paths, Err(BusError::Malformed(_))));
}

#[test]
fn a_cut_message_is_malformed() {
    let bytes = hex("wire/list-sessions-reply");
    for cut in [0, 3, 15, 40, bytes.len().saturating_sub(1)] {
        let short = bytes.get(..cut).unwrap_or_default();
        assert!(matches!(decode(short), Err(BusError::Malformed(_))));
    }
}

#[test]
fn big_endian_or_another_version_is_refused() {
    let bytes = hex("wire/hello-reply");
    for (at, byte) in [(0, b'B'), (3, 2)] {
        let mut changed = bytes.clone();
        if let Some(b) = changed.get_mut(at) {
            *b = byte;
        }
        assert!(matches!(decode(&changed), Err(BusError::Malformed(_))));
    }
    assert!(frame_len(bytes.get(..4).unwrap_or_default()).is_err());
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn no_bytes_panic_the_decoder(bytes in prop::collection::vec(any::<u8>(), 0..256)) {
        let _ = decode(&bytes).map(|m| (m.string(), m.session_paths(), m.variant()));
        let _ = frame_len(&bytes);
    }
}
