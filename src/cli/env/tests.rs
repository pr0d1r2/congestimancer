use std::ffi::OsString;

use super::{Env, distance};
use crate::cli::exit::USAGE;
use crate::cli::table::spec;

fn env(vars: &[(&str, &str)]) -> Env {
    Env::new(
        vars.iter()
            .map(|(k, v)| (OsString::from(k), OsString::from(v))),
    )
}

fn values(env: &Env, name: &str) -> Option<Vec<String>> {
    spec(name).and_then(|s| env.switch(s).ok()).flatten()
}

#[test]
fn a_value_switch_reads_the_same_text_as_the_flag() {
    let vars = env(&[("CGMR_LOAD", "2.5"), ("CGMR_DISK", "5GB")]);
    assert_eq!(values(&vars, "load"), Some(vec!["2.5".to_owned()]));
    assert_eq!(values(&vars, "disk"), Some(vec!["5GB".to_owned()]));
    assert_eq!(values(&vars, "mem"), None);
}

#[test]
fn a_boolean_switch_reads_six_words() {
    for (word, held) in [("1", "true"), ("Yes", "true"), ("no", "false")] {
        let vars = env(&[("CGMR_QUIET", word)]);
        assert_eq!(values(&vars, "quiet"), Some(vec![held.to_owned()]));
    }
    let vars = env(&[("CGMR_COOL", "maybe")]);
    let refused = spec("cool").map(|s| vars.switch(s));
    assert!(matches!(refused, Some(Err(f)) if f.code == USAGE));
}

#[test]
fn a_repeatable_switch_reads_comma_separated_items() {
    let vars = env(&[("CGMR_MAX_PROCS", "rustc*=4, cargo=2,")]);
    let want = vec!["rustc*=4".to_owned(), "cargo=2".to_owned()];
    assert_eq!(values(&vars, "max-procs"), Some(want));
}

#[test]
fn when_never_comes_from_the_environment() {
    let vars = env(&[("CGMR_WHEN", "true"), ("CGMR_WHEN_FILE", "/tmp/go")]);
    assert_eq!(values(&vars, "when"), None);
    assert_eq!(values(&vars, "when-file"), Some(vec!["/tmp/go".to_owned()]));
}

#[test]
fn an_empty_variable_is_unset() {
    assert_eq!(values(&env(&[("CGMR_LOAD", "")]), "load"), None);
}

#[test]
fn an_unknown_variable_names_the_nearest_known_one() {
    let warnings = env(&[("CGMR_LODE", "2"), ("PATH", "/bin")]).unknown();
    assert_eq!(warnings.len(), 1);
    let first = warnings.first().map(String::as_str).unwrap_or_default();
    assert!(first.contains("did you mean CGMR_LOAD?"), "{first}");
}

#[test]
fn a_far_unknown_variable_gets_no_hint() {
    let warnings = env(&[("CGMR_XYZZY_PLUGH", "1")]).unknown();
    let first = warnings.first().map(String::as_str).unwrap_or_default();
    assert_eq!(first, "unknown variable CGMR_XYZZY_PLUGH");
}

#[test]
fn the_variables_cgmr_reads_are_not_warned_about() {
    let known = [
        "CGMR_DOGFOOD",
        "CGMR_FIXTURE_ROOT",
        "CGMR_HELD_SLOTS",
        "CGMR_PROFILE",
        "CGMR_PROFILE_FILE",
        "CGMR_IO_BUSY",
    ];
    let vars: Vec<(&str, &str)> = known.iter().map(|k| (*k, "1")).collect();
    assert_eq!(env(&vars).unknown(), Vec::<String>::new());
}

#[test]
fn cgmr_when_is_named_as_ignored() {
    let warnings = env(&[("CGMR_WHEN", "rm -rf /")]).unknown();
    let first = warnings.first().map(String::as_str).unwrap_or_default();
    assert!(first.starts_with("CGMR_WHEN is ignored"), "{first}");
}

#[test]
fn edit_distance_counts_single_changes() {
    assert_eq!(distance("load", "load"), 0);
    assert_eq!(distance("lode", "load"), 2);
    assert_eq!(distance("", "abc"), 3);
    assert_eq!(distance("kitten", "sitting"), 3);
}

#[test]
fn a_value_that_is_not_utf8_is_a_usage_error() {
    use std::os::unix::ffi::OsStringExt;
    let bad = OsString::from_vec(vec![0xff]);
    let vars = Env::new([(OsString::from("CGMR_LOAD"), bad)]);
    let refused = spec("load").map(|s| vars.switch(s));
    assert!(matches!(refused, Some(Err(f)) if f.code == USAGE));
}
