use std::path::Path;

use super::{PPD_STATE, parse_ppd_profile};
use crate::power::PowerError;

/// The `state.ini` of fixture tree `case`.
fn state(case: &str) -> String {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir
        .join("tests/fixtures/power")
        .join(case)
        .join(PPD_STATE.trim_start_matches('/'));
    std::fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn the_profile_is_read_from_the_state_section() {
    assert_eq!(
        parse_ppd_profile(&state("intel")),
        Ok("balanced".to_owned())
    );
}

#[test]
fn power_saver_is_named_as_saved() {
    assert_eq!(
        parse_ppd_profile(&state("arm")),
        Ok("power-saver".to_owned())
    );
}

#[test]
fn a_profile_outside_the_state_section_does_not_count() {
    let text = "[Other]\nProfile=performance\n[State]\nProfile=balanced\n";
    assert_eq!(parse_ppd_profile(text), Ok("balanced".to_owned()));
}

#[test]
fn no_profile_is_malformed() {
    assert_eq!(
        parse_ppd_profile("[State]\nProfile=\n"),
        Err(PowerError::malformed("state.ini", "[State]\nProfile=\n"))
    );
}
