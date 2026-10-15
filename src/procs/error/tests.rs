use std::error::Error;

use super::ProcsError;
use crate::units::parse_count;

#[test]
fn no_equals_says_the_form() {
    let err = ProcsError::NoEquals {
        input: "cargo".to_owned(),
    };
    assert_eq!(err.to_string(), "`cargo`: expected PATTERN=N, like cargo=4");
    assert!(err.source().is_none());
}

#[test]
fn empty_pattern_says_so() {
    let err = ProcsError::EmptyPattern {
        input: "=4".to_owned(),
    };
    assert_eq!(err.to_string(), "`=4`: the pattern before `=` is empty");
    assert!(err.source().is_none());
}

#[test]
fn count_is_the_unit_error() {
    let unit = parse_count("x").err();
    let err = unit.clone().map(ProcsError::Count);
    assert_eq!(
        err.as_ref().map(ToString::to_string),
        unit.map(|u| u.to_string())
    );
    assert!(err.is_some_and(|e| e.source().is_some()));
}
