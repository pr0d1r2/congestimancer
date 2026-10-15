use std::time::Duration;

use super::{DEFAULT_INTERVAL, MIN_INTERVAL, Pacing};

#[test]
fn the_default_paces_every_two_seconds_and_nothing_else() {
    let pacing = Pacing::default();
    assert_eq!(pacing.interval, Duration::from_secs(2));
    assert_eq!(pacing, Pacing::new(DEFAULT_INTERVAL));
    assert_eq!(pacing.timeout, None);
}

#[test]
fn the_interval_in_use_is_never_below_the_floor() {
    assert_eq!(Pacing::new(Duration::ZERO).base(), MIN_INTERVAL);
    let slow = Duration::from_secs(5);
    assert_eq!(Pacing::new(slow).base(), slow);
}
