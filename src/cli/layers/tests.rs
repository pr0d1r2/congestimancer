use std::ffi::OsString;
use std::path::Path;

use super::{Layer, Resolved, resolve, set_above};
use crate::cli::args::Given;
use crate::cli::env::Env;
use crate::cli::exit::USAGE;
use crate::profile::{Origin, Switch, parse_profiles};
use crate::units::parse_decimal;

fn env(vars: &[(&str, &str)]) -> Env {
    Env::new(
        vars.iter()
            .map(|(k, v)| (OsString::from(k), OsString::from(v))),
    )
}

/// The switches of the one profile in `text`.
fn profile(text: &str) -> Vec<Switch> {
    let file = parse_profiles(text, Path::new("p"), Origin::User);
    let first = file.ok().and_then(|f| f.profiles().first().cloned());
    first.map(|p| p.switches().to_vec()).unwrap_or_default()
}

#[test]
fn the_flag_beats_the_environment_and_the_profile() {
    let given = Given::of(&[("load", "1")]);
    let vars = env(&[("CGMR_LOAD", "2")]);
    let resolved = resolve(&given, &vars, &profile("[p]\n--load 3\n"));
    assert_eq!(
        resolved.ok().as_ref().and_then(|r| r.text("load")),
        Some("1")
    );
}

#[test]
fn the_environment_beats_the_profile() {
    let vars = env(&[("CGMR_LOAD", "2")]);
    let switches = profile("[p]\n--load 3\n--mem 2GB\n");
    let resolved = resolve(&Given::default(), &vars, &switches);
    let resolved = resolved.unwrap_or_default();
    assert_eq!(resolved.text("load"), Some("2"));
    assert_eq!(resolved.text("mem"), Some("2GB"));
}

#[test]
fn a_profile_flag_and_repeats_carry_over() {
    let text = "[p]\n--cool\n--max-procs a=1\n--max-procs b=2\n";
    let resolved = resolve(&Given::default(), &Env::default(), &profile(text));
    let resolved = resolved.unwrap_or_default();
    assert!(resolved.on("cool"));
    assert_eq!(resolved.all("max-procs"), ["a=1", "b=2"]);
}

#[test]
fn an_env_flag_turned_off_is_set_but_not_on() {
    let vars = env(&[("CGMR_VERBOSE", "no")]);
    let resolved = resolve(&Given::default(), &vars, &[]).unwrap_or_default();
    assert!(!resolved.on("verbose"));
    assert!(!resolved.has("verbose"));
    assert!(resolved.text("verbose").is_some());
}

#[test]
fn a_bad_value_names_its_layer() {
    let vars = env(&[("CGMR_LOAD", "high")]);
    let resolved = resolve(&Given::default(), &vars, &[]).unwrap_or_default();
    let refused = resolved.check().err();
    assert_eq!(refused.as_ref().map(|f| f.code), Some(USAGE));
    let message = refused.map(|f| f.message).unwrap_or_default();
    assert!(message.starts_with("CGMR_LOAD: "), "{message}");
}

#[test]
fn parse_reads_the_first_value_or_names_the_flag() {
    let resolved = Resolved::default().with("load", &["x"], Layer::Flag);
    let refused = resolved.parse("load", parse_decimal).err();
    let message = refused.map(|f| f.message).unwrap_or_default();
    assert!(message.starts_with("--load: "), "{message}");
    let none = Resolved::default().parse("load", parse_decimal);
    assert_eq!(none.ok().flatten(), None);
}

#[test]
fn set_above_sees_the_flag_and_the_environment() {
    let given = Given::of(&[("load", "1")]);
    let vars = env(&[("CGMR_MEM", "1GB")]);
    assert!(set_above(&given, &vars, "--load"));
    assert!(set_above(&given, &vars, "--mem"));
    assert!(!set_above(&given, &vars, "--disk"));
    assert!(!set_above(&given, &vars, "--nope"));
}
