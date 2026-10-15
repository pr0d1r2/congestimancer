use super::NetError;
use crate::units::parse_name;

#[test]
fn unknown_iface_names_the_switch_and_the_interface() {
    let err = parse_name("eth9")
        .ok()
        .map(|name| NetError::UnknownIface { name });
    let shown = err.map(|e| e.to_string()).unwrap_or_default();
    assert_eq!(shown, "--net-iface eth9: no such interface");
}
