use std::ffi::OsString;
use std::io;
use std::path::Path;

use super::{CliTable, profile_layer};
use crate::cli::args::Given;
use crate::cli::env::Env;
use crate::cli::exit::USAGE;
use crate::profile::{Arity, SwitchFault, SwitchTable};

const PROFILES: &str = "[heavy]\n--load 1\n--mem 2GB\n--cool\n";

/// A reader that has only `profiles/ci` holding [`PROFILES`].
fn reader(path: &Path) -> io::Result<String> {
    if path == Path::new("profiles/ci") {
        Ok(PROFILES.to_owned())
    } else {
        Err(io::ErrorKind::NotFound.into())
    }
}

fn env(vars: &[(&str, &str)]) -> Env {
    Env::new(
        vars.iter()
            .map(|(k, v)| (OsString::from(k), OsString::from(v))),
    )
}

fn flags(given: &Given, vars: &Env) -> Option<Vec<String>> {
    let layer = profile_layer(given, vars, Path::new("work"), &reader).ok()?;
    Some(layer.iter().map(|s| s.flag().to_owned()).collect())
}

#[test]
fn the_table_checks_profile_lines_as_argv() {
    assert_eq!(CliTable.check("--load", Some("2")), Ok(Arity::Single));
    let procs = CliTable.check("--max-procs", Some("a=1"));
    assert_eq!(procs, Ok(Arity::Repeatable));
    assert_eq!(CliTable.check("--cool", None), Ok(Arity::Single));
    assert_eq!(
        CliTable.check("--nope", Some("1")),
        Err(SwitchFault::Unknown)
    );
    assert_eq!(
        CliTable.check("--profile", Some("x")),
        Err(SwitchFault::Unknown)
    );
}

#[test]
fn the_table_refuses_a_value_out_of_place() {
    let bad = |flag, value| {
        matches!(CliTable.check(flag, value), Err(SwitchFault::BadValue(_)))
    };
    assert!(bad("--cool", Some("1")));
    assert!(bad("--load", None));
    assert!(bad("--load", Some("lots")));
}

#[test]
fn no_profile_asked_means_no_layer() {
    assert_eq!(flags(&Given::default(), &Env::default()), Some(Vec::new()));
}

#[test]
fn the_named_profile_gives_its_switches() {
    let given =
        Given::of(&[("profile", "heavy"), ("profile-file", "profiles/ci")]);
    let want = ["--load", "--mem", "--cool"].map(str::to_owned).to_vec();
    assert_eq!(flags(&given, &Env::default()), Some(want));
}

#[test]
fn the_environment_names_the_profile_and_overrides_its_lines() {
    let vars = env(&[
        ("CGMR_PROFILE", "heavy"),
        ("CGMR_PROFILE_FILE", "profiles/ci"),
        ("CGMR_MEM", "1GB"),
    ]);
    let given = Given::of(&[("load", "3")]);
    assert_eq!(flags(&given, &vars), Some(vec!["--cool".to_owned()]));
}

#[test]
fn an_unknown_profile_is_a_usage_error() {
    let given =
        Given::of(&[("profile", "light"), ("profile-file", "profiles/ci")]);
    let refused =
        profile_layer(&given, &Env::default(), Path::new("w"), &reader);
    assert_eq!(refused.err().map(|f| f.code), Some(USAGE));
    let bad = Given::of(&[("profile", "../x")]);
    let refused = profile_layer(&bad, &Env::default(), Path::new("w"), &reader);
    assert_eq!(refused.err().map(|f| f.code), Some(USAGE));
}
