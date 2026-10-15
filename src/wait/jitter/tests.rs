use std::collections::HashSet;
use std::time::Duration;

use proptest::prelude::*;

use super::jitter_delay;

#[test]
fn no_jitter_is_no_delay() {
    assert_eq!(jitter_delay(42, Duration::ZERO), Duration::ZERO);
}

#[test]
fn the_same_seed_picks_the_same_delay() {
    let max = Duration::from_secs(10);
    assert_eq!(jitter_delay(7, max), jitter_delay(7, max));
}

#[test]
fn neighbouring_seeds_spread_over_the_span() {
    let max = Duration::from_secs(10);
    let picks: HashSet<Duration> =
        (0..100).map(|seed| jitter_delay(seed, max)).collect();
    assert!(picks.len() > 90, "only {} distinct delays", picks.len());
    let low = picks
        .iter()
        .filter(|d| **d < Duration::from_secs(5))
        .count();
    assert!((20..80).contains(&low), "{low} of 100 in the lower half");
}

#[test]
fn a_huge_span_neither_overflows_nor_reaches_the_end() {
    assert!(jitter_delay(u64::MAX, Duration::MAX) < Duration::MAX);
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn every_delay_is_below_the_jitter(seed: u64, nanos in 1..u64::MAX) {
        let max = Duration::from_nanos(nanos);
        prop_assert!(jitter_delay(seed, max) < max);
    }
}
