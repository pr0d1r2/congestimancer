use std::path::PathBuf;

use crate::cli::entry::{Ran, ran};
use crate::cli::remote::ScriptedSsh;

fn cgmr(args: &[&str]) -> Ran {
    ran(args, &[], &mut ScriptedSsh::default())
}

/// A scratch dir private to one test.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("cgmr-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[test]
fn profiles_lists_the_explicit_file() {
    let file = scratch("profiles").join("ci");
    let _ = std::fs::write(&file, "[heavy]\n--load 1\n");
    let path = file.to_string_lossy().into_owned();
    let ran = cgmr(&["profiles", "--profile-file", &path]);
    assert_eq!(ran.code, 0, "{}", ran.err);
    assert!(ran.out.contains("heavy"), "{}", ran.out);
}

#[test]
fn a_profile_with_a_bad_line_is_a_usage_error() {
    let file = scratch("bad-profile").join("ci");
    let _ = std::fs::write(&file, "[heavy]\n--load lots\n");
    let path = file.to_string_lossy().into_owned();
    let args = ["check", "--profile", "heavy", "--profile-file", &path];
    let ran = cgmr(&args);
    assert_eq!(ran.code, 64);
    assert!(ran.err.contains("ci:2"), "{}", ran.err);
}

#[test]
fn a_profile_sets_gates() {
    let file = scratch("gates-profile").join("ci");
    let _ = std::fs::write(&file, "[gone]\n--when-file no/such/file\n");
    let path = file.to_string_lossy().into_owned();
    let args = ["check", "--profile", "gone", "--profile-file", &path];
    assert_eq!(cgmr(&args).code, 1);
}

#[test]
fn completions_come_from_the_switch_table() {
    let ran = cgmr(&["completions", "bash"]);
    assert_eq!(ran.code, 0);
    assert!(ran.out.contains("--io-busy"), "completion lacks --io-busy");
}

#[test]
fn man_prints_roff() {
    let ran = cgmr(&["man"]);
    assert_eq!(ran.code, 0);
    assert!(ran.out.contains(".TH cgmr 1"), "{}", ran.out);
}

#[test]
fn debug_capture_writes_a_fixture_tree() {
    let dir = scratch("capture");
    let path = dir.to_string_lossy().into_owned();
    let ran = cgmr(&["debug", "capture", &path]);
    assert_eq!(ran.code, 0, "{}", ran.err);
}

#[test]
fn debug_capture_into_a_file_fails() {
    let file = scratch("capture-file").join("plain");
    let _ = std::fs::write(&file, "");
    let path = file.to_string_lossy().into_owned();
    assert_eq!(cgmr(&["debug", "capture", &path]).code, 1);
}
