use std::io;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{
    StartLimits, StartRate, admit_start, parse_start_rate, record_start,
};
use crate::slot::LockDir;
use crate::slot::scratch::Scratch;
use crate::units::{Name, UnitError, parse_name};

fn rate(s: &str) -> Option<StartRate> {
    parse_start_rate(s).ok()
}

#[test]
fn a_rate_is_a_count_over_a_duration() {
    let r = rate("4/1m");
    assert_eq!(r.map(|r| r.starts()), Some(4));
    assert_eq!(r.map(|r| r.window()), Some(Duration::from_secs(60)));
}

#[test]
fn a_rate_needs_a_slash_and_both_halves() {
    for bad in ["4", "x/1m", "4/x", "/1m", "4/"] {
        let err = parse_start_rate(bad).err();
        assert!(matches!(err, Some(UnitError::Malformed { .. })), "{bad}");
    }
}

#[test]
fn a_zero_rate_or_window_is_out_of_range() {
    for bad in ["0/1m", "4/0s"] {
        let err = parse_start_rate(bad).err();
        assert!(matches!(err, Some(UnitError::OutOfRange { .. })), "{bad}");
        assert_eq!(err.map(|e| e.input().to_owned()), Some(bad.to_owned()));
    }
}

fn limits(rate_s: Option<&str>, spacing_s: Option<u64>) -> StartLimits {
    StartLimits {
        rate: rate_s.and_then(rate),
        spacing: spacing_s.map(Duration::from_secs),
    }
}

#[test]
fn no_limits_always_admit() {
    let none = StartLimits::default();
    assert!(none.is_none());
    assert_eq!(admit_start(&[5, 6], 7, &none), Some(vec![7]));
}

#[test]
fn a_rate_admits_n_starts_per_window() {
    let two = limits(Some("2/10s"), None);
    assert!(!two.is_none());
    assert_eq!(admit_start(&[], 1000, &two), Some(vec![1000]));
    assert_eq!(admit_start(&[1000], 2000, &two), Some(vec![1000, 2000]));
    assert_eq!(admit_start(&[1000, 2000], 3000, &two), None);
    assert_eq!(admit_start(&[1000, 2000], 10_999, &two), None);
    assert_eq!(
        admit_start(&[1000, 2000], 11_000, &two),
        Some(vec![2000, 11_000])
    );
}

#[test]
fn spacing_keeps_starts_apart() {
    let five = limits(None, Some(5));
    assert_eq!(admit_start(&[1000], 5999, &five), None);
    assert_eq!(admit_start(&[1000], 6000, &five), Some(vec![6000]));
}

#[test]
fn rate_and_spacing_both_hold() {
    let both = limits(Some("3/1m"), Some(10));
    assert_eq!(admit_start(&[0], 5000, &both), None);
    assert_eq!(admit_start(&[0], 10_000, &both), Some(vec![0, 10_000]));
}

#[test]
fn starts_from_the_future_are_dropped() {
    let five = limits(Some("1/1h"), Some(5));
    assert_eq!(admit_start(&[99_000_000], 1000, &five), Some(vec![1000]));
}

fn key(s: &str) -> io::Result<Name> {
    parse_name(s).map_err(|e| io::Error::other(e.to_string()))
}

fn at(secs: u64) -> SystemTime {
    UNIX_EPOCH
        .checked_add(Duration::from_secs(secs))
        .unwrap_or(UNIX_EPOCH)
}

fn start(dir: &LockDir, k: &str, l: &StartLimits, s: u64) -> io::Result<bool> {
    record_start(dir, &key(k)?, l, at(s)).map_err(io::Error::other)
}

#[test]
fn recorded_starts_are_spaced_per_key() -> io::Result<()> {
    let scratch = Scratch::new("starts-record")?;
    let dir = scratch.lock_dir()?;
    let spaced = limits(None, Some(60));
    assert!(start(&dir, "make", &spaced, 1000)?);
    assert!(!start(&dir, "make", &spaced, 1030)?);
    assert!(start(&dir, "cargo", &spaced, 1030)?);
    assert!(start(&dir, "make", &spaced, 1060)?);
    assert!(scratch.path().join("make+starts").is_file());
    Ok(())
}

#[test]
fn recorded_starts_keep_the_rate() -> io::Result<()> {
    let scratch = Scratch::new("starts-rate")?;
    let dir = scratch.lock_dir()?;
    let two = limits(Some("2/1m"), None);
    assert!(start(&dir, "make", &two, 1000)?);
    assert!(start(&dir, "make", &two, 1001)?);
    assert!(!start(&dir, "make", &two, 1002)?);
    assert!(start(&dir, "make", &two, 1060)?);
    Ok(())
}
