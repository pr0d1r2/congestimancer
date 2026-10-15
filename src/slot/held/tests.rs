use super::{HELD_SLOTS_ENV, is_held, mark_held};
use crate::units::{Name, UnitError, parse_name};

fn key(s: &str) -> Result<Name, UnitError> {
    parse_name(s)
}

#[test]
fn the_marker_is_cgmr_held_slots() {
    assert_eq!(HELD_SLOTS_ENV, "CGMR_HELD_SLOTS");
}

#[test]
fn a_key_in_the_marker_is_held() -> Result<(), UnitError> {
    assert!(is_held(Some("cargo,make"), &key("make")?));
    assert!(is_held(Some("make"), &key("make")?));
    Ok(())
}

#[test]
fn a_key_not_in_the_marker_is_not_held() -> Result<(), UnitError> {
    assert!(!is_held(None, &key("make")?));
    assert!(!is_held(Some(""), &key("make")?));
    assert!(!is_held(Some("maker,cmake"), &key("make")?));
    Ok(())
}

#[test]
fn marking_adds_the_key_once() -> Result<(), UnitError> {
    assert_eq!(mark_held(None, &key("make")?), "make");
    assert_eq!(mark_held(Some(""), &key("make")?), "make");
    assert_eq!(mark_held(Some("cargo"), &key("make")?), "cargo,make");
    assert_eq!(mark_held(Some("cargo,make"), &key("make")?), "cargo,make");
    Ok(())
}
