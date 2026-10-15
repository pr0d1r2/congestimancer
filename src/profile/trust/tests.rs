use std::path::{Path, PathBuf};

use crate::profile::{Origin, ProfileError, parse_profiles};

/// The `when.cgmr` fixture: its path and text.
fn fixture() -> (PathBuf, String) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir.join("tests/fixtures/profile/when.cgmr");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    (path, text)
}

/// The line `--when` is refused on when the fixture is read as `origin`,
/// or `None` if it is accepted.
fn refused(origin: Origin) -> Option<usize> {
    let (path, text) = fixture();
    match parse_profiles(&text, &path, origin) {
        Err(ProfileError::Untrusted { at }) => Some(at.line()),
        _ => None,
    }
}

#[test]
fn when_is_allowed_in_the_user_file() {
    assert_eq!(refused(Origin::User), None);
}

#[test]
fn when_in_a_project_file_is_refused_naming_its_line() {
    assert_eq!(refused(Origin::Project), Some(4));
    let (path, text) = fixture();
    let e = parse_profiles(&text, &path, Origin::Project).err();
    let text = e.map(|e| e.to_string()).unwrap_or_default();
    assert!(
        text.contains("when.cgmr:4: --when is allowed only"),
        "{text}"
    );
}

#[test]
fn when_in_an_explicit_file_is_refused() {
    assert_eq!(refused(Origin::Explicit), Some(4));
}

#[test]
fn when_with_an_equals_sign_is_refused_too() {
    let text = "[a]\n--when=true\n";
    let e = parse_profiles(text, Path::new(".cgmr"), Origin::Project);
    assert!(
        matches!(&e, Err(ProfileError::Untrusted { at }) if at.line() == 2),
        "{e:?}"
    );
}

#[test]
fn other_switches_need_no_trust() {
    let text = "[a]\n--whence 1\n--load 2\n";
    let file = parse_profiles(text, Path::new(".cgmr"), Origin::Project);
    assert!(file.is_ok(), "{file:?}");
}
