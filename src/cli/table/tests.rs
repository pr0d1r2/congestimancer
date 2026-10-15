use std::collections::BTreeSet;

use super::{env_name, spec};
use crate::cli::SWITCHES;

/// The human catalogue every switch comes from (src/cli:V7, .:V18).
const CATALOGUE: &str = include_str!("../../../docs/CONGESTION.md");

/// Every `--switch` named in the catalogue's tables.
fn catalogue_switches() -> BTreeSet<String> {
    let rows = CATALOGUE.lines().filter(|l| l.starts_with("| `--"));
    rows.flat_map(|row| row.split(['`', ' ', ',', '|']))
        .filter_map(|word| word.strip_prefix("--"))
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Whether the catalogue names `--name` itself, not a longer switch.
fn mentions(name: &str) -> bool {
    let flag = format!("--{name}");
    CATALOGUE.match_indices(&flag).any(|(at, _)| {
        let rest = CATALOGUE.get(at..).unwrap_or_default();
        let next = rest.chars().nth(flag.len());
        !next.is_some_and(|c| c.is_ascii_alphanumeric() || c == '-')
    })
}

#[test]
fn env_names_are_upper_snake() {
    assert_eq!(env_name("io-busy"), "CGMR_IO_BUSY");
    assert_eq!(env_name("load"), "CGMR_LOAD");
    let load = spec("max-user-procs").and_then(|s| s.env());
    assert_eq!(load.as_deref(), Some("CGMR_MAX_USER_PROCS"));
}

#[test]
fn when_has_no_environment_name() {
    assert_eq!(spec("when").map(|s| s.env()), Some(None));
    let file = spec("when-file").and_then(|s| s.env());
    assert_eq!(file.as_deref(), Some("CGMR_WHEN_FILE"));
}

#[test]
fn names_are_unique() {
    let names: BTreeSet<&str> = SWITCHES.iter().map(|s| s.name()).collect();
    assert_eq!(names.len(), SWITCHES.len());
}

#[test]
fn every_catalogue_switch_is_in_the_table() {
    for name in catalogue_switches() {
        assert!(spec(&name).is_some(), "--{name} is not in SWITCHES");
    }
}

#[test]
fn every_table_switch_is_in_the_catalogue() {
    for s in SWITCHES {
        assert!(mentions(s.name()), "--{} is undocumented", s.name());
    }
}

#[test]
fn only_the_process_gates_repeat() {
    let repeating: Vec<&str> = SWITCHES
        .iter()
        .filter(|s| s.repeats())
        .map(|s| s.name())
        .collect();
    assert_eq!(repeating, ["max-procs", "max-user-procs"]);
}

#[test]
fn flags_take_no_value() {
    let flags = ["cool", "on-ac", "fifo", "govern", "quiet", "verbose"];
    for name in flags {
        assert_eq!(spec(name).map(|s| s.takes_value()), Some(false));
    }
    assert_eq!(spec("load").map(|s| s.takes_value()), Some(true));
    assert!(spec("nope").is_none());
}

#[test]
fn every_switch_has_help() {
    assert!(SWITCHES.iter().all(|s| !s.help().is_empty()));
}
