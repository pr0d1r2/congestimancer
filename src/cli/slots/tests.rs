use std::ffi::OsString;

use super::{lock_dir, plan, reserved, slots};
use crate::cli::env::Env;
use crate::cli::exit::USAGE;
use crate::cli::layers::{Layer, Resolved};
use crate::units::{Bytes, Count};

fn set(pairs: &[(&'static str, &[&str])]) -> Resolved {
    let mut resolved = Resolved::default();
    for (name, values) in pairs {
        resolved = resolved.with(name, values, Layer::Flag);
    }
    resolved
}

fn cmd(words: &[&str]) -> Vec<OsString> {
    words.iter().map(OsString::from).collect()
}

/// An environment whose lock dir is a fresh scratch dir.
fn scratch_env(name: &str) -> Env {
    let dir = std::env::temp_dir()
        .join(format!("cgmr-slots-{name}-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    Env::new([(OsString::from("TMPDIR"), dir.into_os_string())])
}

#[test]
fn no_slot_switch_means_no_plan() {
    let none = plan(&Resolved::default(), &cmd(&["make"]));
    assert_eq!(none.ok().flatten(), None);
}

#[test]
fn slots_key_on_the_command_name() {
    let resolved = set(&[("slots", &["2"]), ("fifo", &["true"])]);
    let planned = plan(&resolved, &cmd(&["/usr/bin/make", "-j8"]));
    let planned = planned.ok().flatten();
    assert_eq!(planned.as_ref().map(|p| p.key.as_str()), Some("make"));
    assert_eq!(planned.as_ref().and_then(|p| p.slots), Some(Count(2)));
    assert_eq!(planned.map(|p| p.fifo), Some(true));
}

#[test]
fn slots_without_a_key_or_command_is_a_usage_error() {
    let refused = plan(&set(&[("slots", &["2"])]), &[]);
    assert_eq!(refused.err().map(|f| f.code), Some(USAGE));
}

#[test]
fn needs_mem_alone_reserves_only_with_a_key() {
    let resolved = set(&[("needs-mem", &["1GB"])]);
    assert_eq!(plan(&resolved, &[]).ok().flatten(), None);
    let planned = plan(&resolved, &cmd(&["make"])).ok().flatten();
    let reserved = planned.and_then(|p| p.needs_mem);
    assert_eq!(reserved, Some(Bytes(1_000_000_000)));
}

#[test]
fn rate_and_spacing_limit_the_starts() {
    let resolved = set(&[
        ("rate", &["4/1m"]),
        ("spacing", &["5s"]),
        ("slot-key", &["k"]),
    ]);
    let planned = plan(&resolved, &[]).ok().flatten();
    let limits = planned.map(|p| p.limits);
    assert!(limits.is_some_and(|l| l.rate.is_some() && l.spacing.is_some()));
}

#[test]
fn reservations_are_read_only_when_needed() {
    assert_eq!(reserved(&Env::default(), false).ok(), Some(Bytes(0)));
    let env = scratch_env("reserved");
    assert_eq!(reserved(&env, true).ok(), Some(Bytes(0)));
    assert!(lock_dir(&env).is_ok());
}

#[test]
fn slots_wait_only_with_a_plan() {
    let env = scratch_env("waiter");
    assert!(slots(&env, None).is_ok());
    let planned = plan(&set(&[("slots", &["1"])]), &cmd(&["make"]));
    assert!(slots(&env, planned.ok().flatten()).is_ok());
}
