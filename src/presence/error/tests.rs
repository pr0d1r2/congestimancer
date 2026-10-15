use std::io;

use super::BusError;

#[test]
fn io_and_malformed_errors_say_d_bus() {
    let io = BusError::from(io::Error::other("broken pipe"));
    assert_eq!(io.to_string(), "D-Bus: broken pipe");
    assert_eq!(BusError::Malformed("short").to_string(), "D-Bus: short");
}

#[test]
fn a_refusal_quotes_the_bus() {
    let auth = BusError::Auth("REJECTED EXTERNAL".to_owned());
    let want = "D-Bus refused EXTERNAL: `REJECTED EXTERNAL`";
    assert_eq!(auth.to_string(), want);
}

#[test]
fn a_remote_error_names_itself() {
    let remote = BusError::Remote {
        name: "org.freedesktop.DBus.Error.AccessDenied".to_owned(),
        message: "no".to_owned(),
    };
    let want = "D-Bus: org.freedesktop.DBus.Error.AccessDenied: no";
    assert_eq!(remote.to_string(), want);
}
