use std::io::ErrorKind;
use std::path::Path;

use super::{linux_interfaces, parse_if_flags, parse_net_dev};
use crate::signal::Root;
use crate::sys::LinkCounters;

/// The fixture tree `tests/fixtures/net/<case>`.
fn root(case: &str) -> Root {
    Root::new(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/net")
            .join(case),
    )
}

/// One direction's counters.
const fn link(bytes: u64, packets: u64, errors: u64) -> LinkCounters {
    LinkCounters {
        bytes,
        packets,
        errors,
    }
}

/// The text of a fixture file.
fn text(case: &str, path: &str) -> String {
    std::fs::read_to_string(root(case).join(path)).unwrap_or_default()
}

/// Name, up and loopback of each interface under the fixture `case`.
fn flags(case: &str) -> Vec<(String, bool, bool)> {
    linux_interfaces(&root(case))
        .unwrap_or_default()
        .into_iter()
        .map(|i| (i.name, i.up, i.loopback))
        .collect()
}

#[test]
fn net_dev_lists_every_interface_with_both_directions() {
    let found =
        parse_net_dev(&text("linux", "/proc/net/dev")).unwrap_or_default();
    let names: Vec<&str> = found.iter().map(|i| i.name.as_str()).collect();
    assert_eq!(names, ["lo", "eth0", "wlan0", "docker0"]);
    assert_eq!(
        found.get(1).map(|i| (i.rx, i.tx)),
        Some((
            link(18_446_744_073_709_551_000, 2751, 3),
            link(1_782_404, 4324, 1)
        ))
    );
}

#[test]
fn net_dev_reads_a_name_glued_to_its_first_counter() {
    let line = "h\nh\n  eth0:42 1 0 0 0 0 0 0 7 2 0 0 0 0 0 0\n";
    let found = parse_net_dev(line).unwrap_or_default();
    assert_eq!(
        found
            .first()
            .map(|i| (i.name.as_str(), i.rx.bytes, i.tx.bytes)),
        Some(("eth0", 42, 7))
    );
}

#[test]
fn net_dev_names_a_torn_line() {
    let err = parse_net_dev(&text("torn", "/proc/net/dev")).err();
    assert_eq!(err.as_ref().map(|e| e.line), Some(3));
    let shown = err.map(|e| e.to_string()).unwrap_or_default();
    assert!(shown.starts_with("/proc/net/dev line 3: "), "{shown}");
}

#[test]
fn net_dev_rejects_a_counter_that_is_not_a_number() {
    let line = "h\nh\n  eth0: x 1 0 0 0 0 0 0 7 2 0 0 0 0 0 0\n";
    assert!(parse_net_dev(line).is_err());
}

#[test]
fn net_dev_rejects_a_line_without_a_colon() {
    let line = "h\nh\n  eth0 1 1 0 0 0 0 0 0 7 2 0 0 0 0 0 0\n";
    assert!(parse_net_dev(line).is_err());
}

#[test]
fn net_dev_with_headers_only_lists_nothing() {
    assert_eq!(parse_net_dev("h\nh\n"), Ok(Vec::new()));
}

#[test]
fn flags_are_0x_hex() {
    assert_eq!(parse_if_flags("0x1003\n"), Some(0x1003));
    assert_eq!(parse_if_flags("0x9"), Some(9));
    assert_eq!(parse_if_flags("up"), None);
    assert_eq!(parse_if_flags("1003"), None);
}

#[test]
fn linux_interfaces_take_up_and_loopback_from_sysfs() {
    let want = [
        ("lo", true, true),
        ("eth0", true, false),
        ("wlan0", true, false),
        ("docker0", false, false),
    ]
    .map(|(n, up, lo)| (n.to_owned(), up, lo));
    assert_eq!(flags("linux"), want);
}

#[test]
fn linux_interface_without_flags_is_up_and_loopback_only_as_lo() {
    let want = [("lo", true, true), ("eth1", true, false)]
        .map(|(n, up, lo)| (n.to_owned(), up, lo));
    assert_eq!(flags("noflags"), want);
}

#[test]
fn linux_interfaces_without_proc_net_dev_are_unsupported() {
    let err = linux_interfaces(&root("absent")).err();
    assert_eq!(err.map(|e| e.kind()), Some(ErrorKind::Unsupported));
}

#[test]
fn linux_interfaces_of_a_torn_file_are_invalid_data() {
    let err = linux_interfaces(&root("torn")).err();
    assert_eq!(err.map(|e| e.kind()), Some(ErrorKind::InvalidData));
}
