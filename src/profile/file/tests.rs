use std::path::{Path, PathBuf};

use super::{Origin, ProfileFile, parse_profiles};
use crate::profile::{At, ProfileError, Switch};
use crate::units::parse_name;

/// A fixture file under `tests/fixtures/profile/`: its path and text.
fn fixture(name: &str) -> (PathBuf, String) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir.join("tests/fixtures/profile").join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    (path, text)
}

/// Parses a fixture as a project file.
fn parse(name: &str) -> Result<ProfileFile, ProfileError> {
    let (path, text) = fixture(name);
    parse_profiles(&text, &path, Origin::Project)
}

/// A switch as owned argv words.
fn words(switch: &Switch) -> Vec<String> {
    switch.args().into_iter().map(str::to_owned).collect()
}

/// The switches of profile `name` in fixture `file`, as argv words.
fn args(file: &str, name: &str) -> Vec<Vec<String>> {
    let parsed = parse(file).ok();
    let name = parse_name(name).ok();
    let profile = parsed.zip(name).and_then(|(f, n)| f.get(&n).cloned());
    let switches = profile.map(|p| p.switches().to_vec()).unwrap_or_default();
    switches.iter().map(words).collect()
}

#[test]
fn sections_hold_switches_as_on_the_command_line() {
    let file = parse("project.cgmr").ok();
    let names: Vec<String> = file
        .iter()
        .flat_map(ProfileFile::profiles)
        .map(|p| p.name().to_string())
        .collect();
    assert_eq!(names, ["build", "ci"]);
    assert_eq!(
        args("project.cgmr", "build"),
        [["--load", "2"], ["--mem", "4GB"]]
    );
}

#[test]
fn one_pair_of_quotes_is_dropped_and_a_bare_switch_has_no_value() {
    assert_eq!(
        args("project.cgmr", "ci"),
        [
            vec!["--path", "target dir"],
            vec!["--path", "cache"],
            vec!["--quiet"],
        ]
    );
}

#[test]
fn every_switch_and_header_keeps_its_line() {
    let file = parse("project.cgmr").ok();
    let build = file.as_ref().and_then(|f| f.profiles().first());
    assert_eq!(build.map(|p| p.at().line()), Some(3));
    let lines: Vec<usize> = build
        .map(|p| p.switches().iter().map(|s| s.at().line()).collect())
        .unwrap_or_default();
    assert_eq!(lines, [4, 5]);
    assert_eq!(
        build.map(|p| p.at().path()),
        file.as_ref().map(ProfileFile::path)
    );
    assert_eq!(file.map(|f| f.origin()), Some(Origin::Project));
}

/// The line and the error text of a fixture that must fail.
fn failure(name: &str) -> (Option<usize>, String) {
    match parse(name) {
        Ok(_) => (None, "parsed".to_owned()),
        Err(e) => (e.at().map(At::line), e.to_string()),
    }
}

#[test]
fn a_switch_before_any_header_names_its_line() {
    let (line, text) = failure("orphan-switch.cgmr");
    assert_eq!(line, Some(2));
    assert!(text.contains("orphan-switch.cgmr:2: "), "{text}");
    assert!(text.contains("before any [name] header"), "{text}");
}

#[test]
fn a_line_that_is_no_switch_names_its_line() {
    let (line, text) = failure("not-a-switch.cgmr");
    assert_eq!(line, Some(2));
    assert!(text.contains("not-a-switch.cgmr:2: "), "{text}");
    assert_eq!(failure("bare-dashes.cgmr").0, Some(2));
}

#[test]
fn an_unclosed_header_names_its_line() {
    let (line, text) = failure("open-header.cgmr");
    assert_eq!(line, Some(1));
    assert!(text.contains("[name]"), "{text}");
}

#[test]
fn a_header_name_is_validated_as_a_name() {
    let e = parse("bad-name.cgmr").err();
    assert!(
        matches!(&e, Some(ProfileError::BadName { at, .. }) if at.line() == 1),
        "{e:?}"
    );
}

#[test]
fn a_name_defined_twice_in_one_file_is_an_error() {
    let e = parse("duplicate.cgmr").err();
    assert!(
        matches!(
            &e,
            Some(ProfileError::DuplicateProfile { at, first: 1, .. })
                if at.line() == 4
        ),
        "{e:?}"
    );
}

#[test]
fn an_empty_text_has_no_profiles() {
    let file = parse_profiles("", Path::new("x"), Origin::User);
    assert_eq!(file.map(|f| f.profiles().len()), Ok(0));
}

#[test]
fn origins_have_words() {
    let words =
        [Origin::Explicit, Origin::Project, Origin::User].map(Origin::as_str);
    assert_eq!(words, ["explicit", "project", "user"]);
}
