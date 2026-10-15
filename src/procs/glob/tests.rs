use proptest::prelude::*;

use super::Glob;

/// Whether `pattern` matches `name`.
fn m(pattern: &str, name: &str) -> bool {
    Glob::new(pattern).matches(name)
}

#[test]
fn literal_matches_only_itself() {
    assert!(m("cargo", "cargo"));
    assert!(!m("cargo", "cargo2"));
    assert!(!m("cargo", "carg"));
    assert!(!m("cargo", "Cargo"));
}

#[test]
fn star_matches_any_run_even_empty() {
    assert!(m("nixbld*", "nixbld"));
    assert!(m("nixbld*", "nixbld12"));
    assert!(m("*rustc*", "rustc"));
    assert!(m("*rustc*", "x-rustc-y"));
    assert!(m("*", ""));
    assert!(!m("nixbld*", "nixbl"));
}

#[test]
fn question_matches_exactly_one() {
    assert!(m("cc?", "cc1"));
    assert!(!m("cc?", "cc"));
    assert!(!m("cc?", "cc12"));
    assert!(m("?", "\u{e9}"));
}

#[test]
fn star_backtracks() {
    assert!(m("a*b*c", "aXbYbZc"));
    assert!(m("*ab", "aab"));
    assert!(!m("a*b", "aXbY"));
    assert!(m("**x", "x"));
}

#[test]
fn empty_pattern_matches_only_empty() {
    assert!(m("", ""));
    assert!(!m("", "a"));
}

#[test]
fn renders_as_given() {
    assert_eq!(Glob::new("nix*").to_string(), "nix*");
    assert_eq!(Glob::new("nix*").as_str(), "nix*");
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn plain_name_matches_itself(name in "[a-z0-9._-]{0,16}") {
        prop_assert!(m(&name, &name));
    }

    #[test]
    fn star_suffix_matches_any_tail(
        head in "[a-z]{0,8}",
        tail in "[a-z?*]{0,8}",
    ) {
        let pattern = format!("{head}*");
        let name = format!("{head}{tail}");
        prop_assert!(m(&pattern, &name));
    }
}
