use std::time::Duration;

use super::{Session, idle, logind_idle};
use crate::presence::error::BusError;
use crate::presence::fixture::{Duplex, hex};

/// Now, in microseconds since the epoch.
const NOW: u64 = 1_700_000_000_000_000;

/// A local user session of `kind`, idle hint `hint`, since `ago` s.
fn session(kind: &str, hint: bool, ago: u64) -> Session {
    Session {
        class: "user".to_owned(),
        remote: false,
        kind: kind.to_owned(),
        idle_hint: hint,
        idle_since_us: NOW.saturating_sub(ago.saturating_mul(1_000_000)),
    }
}

#[test]
fn a_desktop_is_idle_only_when_it_says_so() {
    let idle_x11 = session("x11", true, 90);
    assert_eq!(idle(&[idle_x11], NOW), Some(Duration::from_secs(90)));
    let busy = session("wayland", false, 90);
    assert_eq!(idle(&[busy], NOW), Some(Duration::ZERO));
}

#[test]
fn a_tty_is_idle_since_its_atime() {
    let tty = session("tty", false, 300);
    assert_eq!(idle(&[tty], NOW), Some(Duration::from_secs(300)));
}

#[test]
fn the_least_idle_session_wins() {
    let sessions = [session("tty", false, 300), session("x11", true, 30)];
    assert_eq!(idle(&sessions, NOW), Some(Duration::from_secs(30)));
}

#[test]
fn remote_and_non_user_sessions_do_not_count() {
    let mut ssh = session("tty", false, 1);
    ssh.remote = true;
    let mut greeter = session("x11", true, 2);
    greeter.class = "greeter".to_owned();
    assert_eq!(idle(&[ssh, greeter], NOW), None);
    assert_eq!(idle(&[], NOW), None);
}

#[test]
fn a_hint_from_the_future_is_no_idle() {
    let ahead = session("x11", true, 0);
    assert_eq!(idle(&[ahead], NOW.saturating_sub(5)), Some(Duration::ZERO));
}

#[test]
fn a_desktop_bus_gives_its_one_local_user_session() {
    let mut duplex = Duplex::serving("stream-desktop");
    let got = logind_idle(&mut duplex, 1000, NOW);
    assert_eq!(got, Ok(Some(Duration::from_secs(60))));
    let first_get = hex("wire/get-class-call");
    let sent = duplex.output.windows(first_get.len());
    assert!(sent.into_iter().any(|w| w == first_get.as_slice()));
}

#[test]
fn a_bus_that_hangs_up_mid_session_is_the_error() {
    // The desktop stream cut right after the ListSessions reply.
    let parts = ["wire/hello-reply", "wire/name-acquired"];
    let lens = parts.into_iter().chain(["wire/list-sessions-reply"]);
    let cut = lens.map(|name| hex(name).len()).sum::<usize>();
    let mut bytes = hex("stream-desktop");
    bytes.truncate(cut.saturating_add(37));
    let err = logind_idle(Duplex::from_bytes(bytes), 1000, NOW);
    assert!(matches!(err, Err(BusError::Io(_))), "{err:?}");
}

#[test]
fn a_headless_bus_gives_no_idle() {
    let mut duplex = Duplex::serving("stream-headless");
    assert_eq!(logind_idle(&mut duplex, 1000, NOW), Ok(None));
}

#[test]
fn a_refused_login_is_the_error() {
    let mut duplex = Duplex::serving("stream-rejected");
    let err = logind_idle(&mut duplex, 1000, NOW);
    assert_eq!(err, Err(BusError::Auth("REJECTED EXTERNAL".to_owned())));
}
