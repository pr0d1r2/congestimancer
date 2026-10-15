use std::path::PathBuf;

use super::{SYSTEM_BUS_SOCKET, system_bus_path};

fn path(text: &str) -> Option<PathBuf> {
    Some(PathBuf::from(text))
}

#[test]
fn unset_means_the_default_socket() {
    assert_eq!(system_bus_path(None), path(SYSTEM_BUS_SOCKET));
}

#[test]
fn a_unix_path_entry_is_used() {
    let address = "unix:path=/var/run/dbus/system_bus_socket";
    assert_eq!(
        system_bus_path(Some(address)),
        path("/var/run/dbus/system_bus_socket")
    );
}

#[test]
fn other_keys_and_entries_are_skipped() {
    let address = "tcp:host=localhost,port=1;unix:guid=ab,path=/tmp/bus";
    assert_eq!(system_bus_path(Some(address)), path("/tmp/bus"));
}

#[test]
fn escapes_in_the_path_are_decoded() {
    let address = "unix:path=/tmp/my%20bus";
    assert_eq!(system_bus_path(Some(address)), path("/tmp/my bus"));
}

#[test]
fn no_unix_path_is_none() {
    assert_eq!(system_bus_path(Some("unix:abstract=/tmp/x")), None);
    assert_eq!(system_bus_path(Some("tcp:host=h")), None);
    assert_eq!(system_bus_path(Some("unix:path=/tmp/bad%zz")), None);
    assert_eq!(system_bus_path(Some("")), None);
}

#[cfg(not(target_os = "linux"))]
#[test]
fn other_oses_are_unsupported() {
    let err = super::system_bus_connect().err().map(|e| e.kind());
    assert_eq!(err, Some(std::io::ErrorKind::Unsupported));
}
