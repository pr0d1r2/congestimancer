use std::time::{Duration, Instant};

use super::{Run, beat_after, earliest};
use crate::wait::{MIN_INTERVAL, Pacing};

const SEC: Duration = Duration::from_secs(1);

fn at(start: Instant, secs: u64) -> Instant {
    start
        .checked_add(Duration::from_secs(secs))
        .unwrap_or(start)
}

#[test]
fn earliest_picks_the_first_instant_or_none() {
    let t = Instant::now();
    assert_eq!(earliest(None, []), None);
    assert_eq!(earliest(Some(at(t, 5)), [at(t, 3)]), Some(at(t, 3)));
    assert_eq!(earliest(None, [at(t, 4), at(t, 2)]), Some(at(t, 2)));
    assert_eq!(earliest(Some(at(t, 1)), [at(t, 2)]), Some(at(t, 1)));
}

#[test]
fn a_zero_heartbeat_still_waits_the_floor() {
    let t = Instant::now();
    let mut pacing = Pacing::new(SEC);
    assert_eq!(beat_after(&pacing, t), None);
    pacing.heartbeat = Some(Duration::ZERO);
    assert_eq!(beat_after(&pacing, t), t.checked_add(MIN_INTERVAL));
}

#[test]
fn a_fresh_run_is_due_and_pauses_the_interval() {
    let t = Instant::now();
    let pacing = Pacing::new(SEC);
    let run = Run::new(&pacing, t);
    assert!(run.due(t));
    assert!(!run.expired(t));
    assert_eq!(run.pause(t), SEC);
}

#[test]
fn a_run_past_its_deadline_pauses_the_interval_not_zero() {
    let t = Instant::now();
    let mut pacing = Pacing::new(SEC);
    pacing.timeout = Some(SEC);
    let run = Run::new(&pacing, t);
    assert!(run.expired(at(t, 2)));
    assert_eq!(run.pause(at(t, 2)), SEC);
}
