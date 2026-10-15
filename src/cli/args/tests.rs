use std::ffi::OsString;

use clap::error::ErrorKind;

use super::{Cli, DebugVerb, Given, Verb, parse, with_default_verb};

fn argv(words: &[&str]) -> Vec<OsString> {
    words.iter().map(OsString::from).collect()
}

fn verb(words: &[&str]) -> Option<Verb> {
    parse(argv(words)).ok().map(|cli: Cli| cli.verb)
}

fn run_given(words: &[&str]) -> Option<Given> {
    match verb(words) {
        Some(Verb::Run(run)) => Some(run.given),
        _ => None,
    }
}

fn error(words: &[&str]) -> Option<ErrorKind> {
    parse(argv(words)).err().map(|e| e.kind())
}

#[test]
fn no_verb_means_run() {
    let given = run_given(&["cgmr", "--load", "2"]);
    assert_eq!(given, Some(Given::of(&[("load", "2")])));
    assert_eq!(run_given(&["cgmr"]), Some(Given::default()));
}

#[test]
fn default_verb_goes_before_the_switches() {
    let args = with_default_verb(argv(&["cgmr", "--load", "check"]));
    assert_eq!(args, argv(&["cgmr", "run", "--load", "check"]));
    let named = with_default_verb(argv(&["cgmr", "status"]));
    assert_eq!(named, argv(&["cgmr", "status"]));
    assert_eq!(with_default_verb(Vec::new()), argv(&["run"]));
}

#[test]
fn the_report_verbs_parse() {
    let check = verb(&["cgmr", "check", "--json"]);
    assert!(matches!(check, Some(Verb::Check(r)) if r.json));
    assert!(matches!(verb(&["cgmr", "status"]), Some(Verb::Status(_))));
}

#[test]
fn advise_takes_jobs() {
    let advise = verb(&["cgmr", "advise", "--jobs", "8"]);
    assert!(matches!(advise, Some(Verb::Advise(a)) if a.jobs == Some(8)));
}

#[test]
fn the_other_verbs_parse() {
    let profiles = verb(&["cgmr", "profiles"]);
    assert!(matches!(profiles, Some(Verb::Profiles(_))));
    let zsh = verb(&["cgmr", "completions", "zsh"]);
    assert!(matches!(zsh, Some(Verb::Completions { .. })));
    assert!(matches!(verb(&["cgmr", "man"]), Some(Verb::Man)));
}

#[test]
fn debug_capture_is_a_hidden_verb() {
    let capture = verb(&["cgmr", "debug", "capture", "out"]);
    let dir = match capture {
        Some(Verb::Debug(DebugVerb::Capture { dir })) => Some(dir),
        _ => None,
    };
    assert_eq!(dir, Some("out".into()));
}

#[test]
fn the_command_comes_only_after_the_separator() {
    let cmd = match verb(&["cgmr", "--load", "2", "--", "make", "-j8"]) {
        Some(Verb::Run(run)) => run.cmd,
        _ => Vec::new(),
    };
    assert_eq!(cmd, argv(&["make", "-j8"]));
    assert!(error(&["cgmr", "make"]).is_some());
}

#[test]
fn a_single_value_switch_given_twice_is_an_error() {
    let twice = error(&["cgmr", "--load", "1", "--load", "2"]);
    assert_eq!(twice, Some(ErrorKind::ArgumentConflict));
    assert!(error(&["cgmr", "check", "--quiet", "--quiet"]).is_some());
}

#[test]
fn the_process_gates_repeat() {
    let words = ["cgmr", "--max-procs", "rustc*=4", "--max-procs", "cargo=2"];
    let want =
        Given::of(&[("max-procs", "rustc*=4"), ("max-procs", "cargo=2")]);
    assert_eq!(run_given(&words), Some(want));
}

#[test]
fn a_flag_is_held_as_true() {
    let given = run_given(&["cgmr", "--cool", "--on-ac"]);
    let want = Given::of(&[("cool", "true"), ("on-ac", "true")]);
    assert_eq!(given, Some(want));
}

#[test]
fn an_unknown_switch_is_an_error() {
    assert_eq!(
        error(&["cgmr", "--lode", "2"]),
        Some(ErrorKind::UnknownArgument)
    );
}
