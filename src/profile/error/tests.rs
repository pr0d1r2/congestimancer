use std::path::Path;

use super::{At, ProfileError};
use crate::units::{Name, parse_name};

/// Line 3 of `p/.cgmr`.
fn at() -> At {
    At::new("p/.cgmr", 3)
}

/// A name from a literal known to be one.
fn name(s: &str) -> Name {
    parse_name(s).unwrap_or_else(|_| name("bad-literal"))
}

/// What an error says after its `file:line: `, or `None` if it does not
/// start with line 3 of `p/.cgmr`.
fn said(e: &ProfileError) -> Option<String> {
    let text = e.to_string();
    text.strip_prefix("p/.cgmr:3: ").map(str::to_owned)
}

#[test]
fn a_place_renders_as_file_colon_line() {
    assert_eq!(at().to_string(), "p/.cgmr:3");
    assert_eq!(at().path(), Path::new("p/.cgmr"));
    assert_eq!(at().line(), 3);
}

#[test]
fn every_error_is_a_usage_error() {
    assert_eq!(ProfileError::EXIT, 64);
}

#[test]
fn file_errors_start_with_file_and_line() {
    let syntax = ProfileError::Syntax {
        at: at(),
        reason: "not a switch",
    };
    assert_eq!(said(&syntax).as_deref(), Some("not a switch"));
    let dup = ProfileError::DuplicateProfile {
        at: at(),
        name: name("build"),
        first: 1,
    };
    let text = said(&dup);
    assert_eq!(
        text.as_deref(),
        Some("profile [build] already defined on line 1")
    );
}

#[test]
fn a_bad_header_name_says_why() {
    let bad = parse_name("a b")
        .err()
        .map(|error| ProfileError::BadName { at: at(), error });
    let text = bad.as_ref().and_then(said).unwrap_or_default();
    assert!(text.starts_with("bad profile name: `a b`"), "{text}");
}

#[test]
fn switch_errors_name_the_switch() {
    let value = ProfileError::BadValue {
        at: at(),
        flag: "--load".to_owned(),
        reason: "bad".to_owned(),
    };
    assert_eq!(said(&value).as_deref(), Some("--load: bad"));
    let repeated = ProfileError::RepeatedSwitch {
        at: at(),
        flag: "--load".to_owned(),
        first: 2,
    };
    let text = said(&repeated);
    assert_eq!(text.as_deref(), Some("--load already set on line 2"));
}

#[test]
fn an_untrusted_when_says_where_it_is_allowed() {
    let text = said(&ProfileError::Untrusted { at: at() });
    let text = text.unwrap_or_default();
    assert!(
        text.starts_with("--when is allowed only in the user"),
        "{text}"
    );
}

#[test]
fn a_read_error_names_the_file() {
    let e = ProfileError::Read {
        path: "p/.cgmr".into(),
        reason: "permission denied".to_owned(),
    };
    assert_eq!(e.at(), None);
    assert_eq!(e.to_string(), "cannot read p/.cgmr: permission denied");
}

/// The text for an unknown `bild` with these names defined.
fn unknown(available: &[&str]) -> String {
    let e = ProfileError::UnknownProfile {
        name: name("bild"),
        available: available.iter().copied().map(name).collect(),
    };
    e.to_string()
}

#[test]
fn an_unknown_profile_lists_the_names_there_are() {
    let some = unknown(&["build", "ci"]);
    assert_eq!(some, "unknown profile bild; available: build, ci");
    let none = unknown(&[]);
    assert_eq!(none, "unknown profile bild: no profiles are defined");
}
