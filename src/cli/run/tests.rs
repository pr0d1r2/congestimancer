use std::time::Duration;

use super::{explain, pacing, priority, seed};
use crate::cli::entry::ran;
use crate::cli::env::Env;
use crate::cli::layers::{Layer, Resolved};
use crate::cli::remote::ScriptedSsh;
use crate::cli::world::World;
use crate::gate::{Evaluation, Overall};
use crate::launch::Priority;
use crate::wait::Outcome;

fn set(pairs: &[(&'static str, &[&'static str])]) -> Resolved {
    let mut resolved = Resolved::default();
    for (name, values) in pairs {
        resolved = resolved.with(name, values, Layer::Flag);
    }
    resolved
}

#[test]
fn priority_follows_the_switches() {
    assert_eq!(priority(&Resolved::default()), Priority::Normal);
    let or = set(&[("or-background", &["true"])]);
    assert_eq!(priority(&or), Priority::OrBackground);
    let both = set(&[("or-background", &["true"]), ("background", &["true"])]);
    assert_eq!(priority(&both), Priority::Background);
}

/// Every timing switch, set.
const TIMING: [(&str, &[&str]); 7] = [
    ("interval", &["1s"]),
    ("backoff", &["8s"]),
    ("timeout", &["1m"]),
    ("settle", &["3s"]),
    ("jitter", &["2s"]),
    ("heartbeat", &["30s"]),
    ("verbose", &["true"]),
];

#[test]
fn pacing_reads_every_timing_switch() {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let mut ssh = ScriptedSsh::default();
    let world = World::new(Env::default(), &mut out, &mut err, &mut ssh);
    let pacing = pacing(&world, &set(&TIMING)).ok();
    let timeout = pacing.and_then(|p| p.timeout);
    assert_eq!(timeout, Some(Duration::from_secs(60)));
    let heartbeat = pacing.and_then(|p| p.heartbeat);
    assert_eq!(heartbeat, Some(Duration::from_secs(30)));
    assert!(pacing.is_some_and(|p| p.verbose.is_some() && p.settle.is_some()));
}

#[test]
fn explain_names_the_unsupported_gates() {
    let mut err = Vec::new();
    let eval = Evaluation {
        verdicts: Vec::new(),
        overall: Overall::Unsupported,
    };
    explain(&mut err, &Outcome::Unsupported(eval));
    assert!(err.is_empty());
    explain(&mut err, &Outcome::TimedOut(None));
    assert_eq!(String::from_utf8_lossy(&err), "cgmr: --timeout ran out: \n");
}

#[test]
fn seeds_differ_between_calls() {
    let first = seed();
    std::thread::sleep(Duration::from_millis(1));
    assert_ne!(first, seed());
}

#[test]
fn a_slot_without_a_key_is_a_usage_error() {
    let ran = ran(&["--slots", "1"], &[], &mut ScriptedSsh::default());
    assert_eq!(ran.code, 64);
    assert!(ran.err.contains("--slot-key"), "{}", ran.err);
}
