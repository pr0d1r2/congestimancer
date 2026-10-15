use super::slot_key;
use crate::slot::SlotError;

fn key(explicit: Option<&str>, cmd: Option<&str>) -> Option<String> {
    slot_key(explicit, cmd).ok().map(|k| k.as_str().to_owned())
}

#[test]
fn an_explicit_key_wins_over_the_command() {
    assert_eq!(
        key(Some("build"), Some("/usr/bin/make")),
        Some("build".into())
    );
}

#[test]
fn an_explicit_key_must_be_a_name() {
    let err = slot_key(Some("../etc"), Some("make")).err();
    assert_eq!(
        err,
        Some(SlotError::BadKey {
            input: "../etc".to_owned()
        })
    );
}

#[test]
fn the_default_key_is_the_command_basename() {
    assert_eq!(key(None, Some("/usr/bin/cargo")), Some("cargo".into()));
    assert_eq!(key(None, Some("make")), Some("make".into()));
}

#[test]
fn unsafe_characters_in_the_basename_become_underscores() {
    assert_eq!(key(None, Some("./my tool+x")), Some("my_tool_x".into()));
    assert_eq!(key(None, Some("-weird")), Some("_weird".into()));
    assert_eq!(
        key(None, Some("za\u{17c}\u{f3}\u{142}\u{107}")),
        Some("za____".into())
    );
}

#[test]
fn a_long_basename_is_cut_to_64() {
    let long = "a".repeat(100);
    assert_eq!(key(None, Some(&long)).map(|k| k.len()), Some(64));
}

#[test]
fn a_basename_that_makes_no_name_is_refused() {
    assert!(key(None, Some("..")).is_none());
    assert!(key(None, Some("/")).is_none());
    assert!(key(None, Some("")).is_none());
}

#[test]
fn no_key_and_no_command_is_refused() {
    let err = slot_key(None, None).err();
    assert_eq!(
        err,
        Some(SlotError::BadKey {
            input: String::new()
        })
    );
}
