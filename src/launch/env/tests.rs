use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::process::Command;

use super::clean_env;
use crate::slot::HELD_SLOTS_ENV;

fn vars(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    pairs
        .iter()
        .map(|&(k, v)| (OsString::from(k), OsString::from(v)))
        .collect()
}

/// What `command` changes: `None` is a removal.
fn changes(command: &Command) -> BTreeMap<String, Option<String>> {
    command
        .get_envs()
        .map(|(k, v)| {
            let text = |s: &OsStr| s.to_string_lossy().into_owned();
            (text(k), v.map(text))
        })
        .collect()
}

#[test]
fn removes_every_cgmr_default() {
    let mut command = Command::new("true");
    let env = vars(&[
        ("CGMR_LOAD", "2"),
        ("CGMR_TIMEOUT", "1h"),
        ("PATH", "/bin"),
        ("XCGMR_LOAD", "keep"),
    ]);
    clean_env(&mut command, env, None);
    let want = BTreeMap::from([
        ("CGMR_LOAD".to_owned(), None),
        ("CGMR_TIMEOUT".to_owned(), None),
    ]);
    assert_eq!(changes(&command), want);
}

#[test]
fn keeps_the_inherited_held_marker_when_nothing_new_is_held() {
    let mut command = Command::new("true");
    let env = vars(&[(HELD_SLOTS_ENV, "build"), ("CGMR_SLOTS", "2")]);
    clean_env(&mut command, env, None);
    let want = BTreeMap::from([("CGMR_SLOTS".to_owned(), None)]);
    assert_eq!(changes(&command), want);
}

#[test]
fn sets_the_held_marker_to_the_keys_now_held() {
    let mut command = Command::new("true");
    let env = vars(&[(HELD_SLOTS_ENV, "build")]);
    clean_env(&mut command, env, Some("build,test"));
    let want = BTreeMap::from([(
        HELD_SLOTS_ENV.to_owned(),
        Some("build,test".to_owned()),
    )]);
    assert_eq!(changes(&command), want);
}

#[test]
fn the_command_sees_the_cleaned_environment() -> std::io::Result<()> {
    let mut command = Command::new("/bin/sh");
    command.args([
        "-c",
        "test -z \"$CGMR_LOAD\" && test \"$CGMR_HELD_SLOTS\" = k",
    ]);
    command.env("CGMR_LOAD", "2");
    clean_env(&mut command, vars(&[("CGMR_LOAD", "2")]), Some("k"));
    assert!(command.status()?.success());
    Ok(())
}

#[test]
fn export_jobs_splits_the_cpus_across_the_holders() {
    let mut command = Command::new("true");
    let names = ["CARGO_BUILD_JOBS".to_owned(), "NIX_BUILD_CORES".to_owned()];
    super::export_jobs(&mut command, &names, 8, 2);
    let want = BTreeMap::from([
        ("CARGO_BUILD_JOBS".to_owned(), Some("4".to_owned())),
        ("NIX_BUILD_CORES".to_owned(), Some("4".to_owned())),
    ]);
    assert_eq!(changes(&command), want);
}

#[test]
fn export_jobs_never_goes_below_one() {
    let names = ["J".to_owned()];
    for (threads, holders, want) in [(2, 8, "1"), (0, 1, "1"), (4, 0, "4")] {
        let mut command = Command::new("true");
        super::export_jobs(&mut command, &names, threads, holders);
        let share = changes(&command).remove("J").flatten();
        assert_eq!(share.as_deref(), Some(want));
    }
}

#[test]
fn export_jobs_leaves_the_held_slot_marker_alone() {
    let mut command = Command::new("true");
    command.env(HELD_SLOTS_ENV, "k");
    let names = [HELD_SLOTS_ENV.to_owned(), "J".to_owned()];
    super::export_jobs(&mut command, &names, 8, 2);
    let want = BTreeMap::from([
        (HELD_SLOTS_ENV.to_owned(), Some("k".to_owned())),
        ("J".to_owned(), Some("4".to_owned())),
    ]);
    assert_eq!(changes(&command), want);
}
