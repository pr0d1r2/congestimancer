use std::time::Duration;

use super::{Clock, SystemClock};
use crate::signal::ManualClock;

#[test]
fn the_system_clock_never_goes_back() {
    let mut clock = SystemClock;
    let before = clock.now();
    clock.sleep(Duration::from_millis(1));
    let after = clock.now();
    assert!(after.duration_since(before) >= Duration::from_millis(1));
}

#[test]
fn a_manual_clock_moves_only_when_told() {
    let mut clock = ManualClock::new();
    let start = clock.now();
    assert_eq!(clock.now(), start);
    clock.advance(Duration::from_secs(2));
    assert_eq!(clock.now().duration_since(start), Duration::from_secs(2));
}

#[test]
fn a_manual_sleep_returns_at_once_and_advances() {
    let mut clock = ManualClock::new();
    let start = clock.now();
    clock.sleep(Duration::from_secs(5));
    clock.sleep(Duration::from_millis(250));
    let slept = clock.now().duration_since(start);
    assert_eq!(slept, Duration::from_millis(5250));
    assert_eq!(
        clock.sleeps(),
        [Duration::from_secs(5), Duration::from_millis(250)]
    );
}
