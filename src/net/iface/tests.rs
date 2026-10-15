use super::parse_net_iface;
use crate::net::NetError;

#[test]
fn an_interface_name_parses_as_written() {
    for s in ["en0", "eth0", "wlp3s0", "br-1a2b", "enp0s31f6.100"] {
        let parsed = parse_net_iface(s).map(|n| n.as_str().to_owned());
        assert_eq!(parsed, Ok(s.to_owned()));
    }
}

#[test]
fn a_name_that_could_leave_sys_class_net_is_a_usage_error() {
    for s in ["", "..", "../eth0", "eth0/flags", "-x", "eth 0"] {
        let err = parse_net_iface(s).err();
        assert!(matches!(err, Some(NetError::BadIface { .. })), "{s:?}");
    }
}

#[test]
fn a_bad_name_names_the_switch_and_the_input() {
    let shown = parse_net_iface("a/b").err().map(|e| e.to_string());
    let shown = shown.unwrap_or_default();
    assert!(shown.starts_with("--net-iface: `a/b` is "), "{shown}");
}
