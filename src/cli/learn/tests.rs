use std::ffi::OsString;

use super::{Need, apply, parse_need};
use crate::cli::env::Env;
use crate::cli::exit::USAGE;
use crate::cli::layers::{Layer, Resolved};
use crate::slot::{LockDir, record_peak};
use crate::sys::current_uid;
use crate::units::{Bytes, parse_name};

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

/// An environment whose lock dir is a fresh scratch dir, and that dir.
fn scratch(name: &str) -> (Env, LockDir) {
    let dir = std::env::temp_dir()
        .join(format!("cgmr-learn-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(&dir);
    let env = Env::new([(OsString::from("TMPDIR"), dir.clone().into())]);
    let lock = LockDir::open(dir.join("cgmr"), current_uid());
    (env, lock.unwrap_or_else(|e| unreachable!("{e}")))
}

fn learn(env: &Env, resolved: &mut Resolved, words: &[&str]) -> Option<String> {
    let learning = apply(env, resolved, &cmd(words)).ok().flatten();
    learning.map(|l| l.name().as_str().to_owned())
}

#[test]
fn a_size_is_a_fixed_need() {
    assert_eq!(parse_need("2GiB"), Ok(Need::Fixed(Bytes(2 << 30))));
}

#[test]
fn auto_takes_an_optional_fallback() {
    assert_eq!(parse_need("auto"), Ok(Need::Auto(None)));
    assert_eq!(
        parse_need("auto:2GiB"),
        Ok(Need::Auto(Some(Bytes(2 << 30))))
    );
}

#[test]
fn a_bad_fallback_or_word_is_refused_in_words() {
    assert!(parse_need("auto:lots").is_err());
    assert!(parse_need("auto:").is_err());
    assert!(parse_need("automatic").is_err());
}

#[test]
fn a_fixed_need_is_left_alone_and_learns_nothing() {
    let (env, _) = scratch("fixed");
    let mut resolved = set(&[("needs-mem", &["2GB"])]);
    assert_eq!(learn(&env, &mut resolved, &["make"]), None);
    assert_eq!(resolved.text("needs-mem"), Some("2GB"));
}

#[test]
fn an_unknown_key_reserves_the_fallback() {
    let (env, _) = scratch("fallback");
    let mut resolved = set(&[("needs-mem", &["auto:2GiB"])]);
    assert_eq!(learn(&env, &mut resolved, &["make"]), Some("make".into()));
    assert_eq!(resolved.text("needs-mem"), Some("2147483648B"));
}

#[test]
fn an_unknown_key_without_a_fallback_reserves_a_gibibyte() {
    let (env, _) = scratch("default");
    let mut resolved = set(&[("needs-mem", &["auto"])]);
    learn(&env, &mut resolved, &["make"]);
    assert_eq!(resolved.text("needs-mem"), Some("1073741824B"));
}

#[test]
fn a_learned_key_reserves_its_peak_and_a_tenth() -> Result<(), String> {
    let (env, lock) = scratch("learned");
    let key = parse_name("gate").map_err(|e| e.to_string())?;
    record_peak(&lock, &key, Bytes(1_000)).map_err(|e| e.to_string())?;
    let mut resolved =
        set(&[("needs-mem", &["auto:5GB"]), ("mem-key", &["gate"])]);
    assert_eq!(learn(&env, &mut resolved, &["make"]), Some("gate".into()));
    assert_eq!(resolved.text("needs-mem"), Some("1100B"));
    Ok(())
}

#[test]
fn the_mem_key_falls_back_to_the_slot_key_then_the_command() {
    let (env, _) = scratch("keys");
    let mut by_slot = set(&[("needs-mem", &["auto"]), ("slot-key", &["s"])]);
    assert_eq!(learn(&env, &mut by_slot, &["make"]), Some("s".into()));
    let mut by_cmd = set(&[("needs-mem", &["auto"])]);
    assert_eq!(
        learn(&env, &mut by_cmd, &["/bin/make"]),
        Some("make".into())
    );
}

#[test]
fn no_command_and_no_key_reserves_the_fallback_and_learns_nothing() {
    let (env, _) = scratch("nokey");
    let mut resolved = set(&[("needs-mem", &["auto:3GB"])]);
    assert_eq!(learn(&env, &mut resolved, &[]), None);
    assert_eq!(resolved.text("needs-mem"), Some("3000000000B"));
}

#[test]
fn a_bad_mem_key_is_a_usage_error() {
    let (env, _) = scratch("badkey");
    let mut resolved = set(&[("needs-mem", &["auto"]), ("mem-key", &["a/b"])]);
    let refused = apply(&env, &mut resolved, &cmd(&["make"]));
    assert_eq!(refused.err().map(|f| f.code), Some(USAGE));
}
