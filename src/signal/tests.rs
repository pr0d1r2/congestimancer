//! Rules that hold across every input a signal's parts take.

use std::time::Duration;

use proptest::prelude::*;

use super::{
    Clock, Counter, ManualClock, Reading, Sampler, SignalId, parse_psi,
};

/// Proptest without a regressions file in the tree.
fn config() -> ProptestConfig {
    ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    }
}

/// Text built from the pressure grammar's own words, so cases reach past
/// the first token.
const NEAR: &str = "((some|full|half)( avg(10|60|300)=[0-9.-]{0,7}| \
                    total=[0-9+-]{0,22}| ){0,6}\n?){0,3}";

/// Two samples of one rate counter, `ms` apart.
fn two_rates(a: u64, b: u64, ms: u64) -> (Reading, Reading) {
    let mut clock = ManualClock::new();
    let mut sampler = Sampler::new();
    let first =
        sampler.update(SignalId::Io, Counter::Rate { total: a }, clock.now());
    clock.advance(Duration::from_millis(ms));
    let second =
        sampler.update(SignalId::Io, Counter::Rate { total: b }, clock.now());
    (first, second)
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn the_psi_parser_never_panics_on_any_text(s in any::<String>()) {
        let _ = parse_psi(&s);
    }

    #[test]
    fn the_psi_parser_never_panics_near_its_grammar(s in NEAR) {
        let _ = parse_psi(&s);
    }

    #[test]
    fn a_first_sample_always_warms_up(a in any::<u64>(), b in any::<u64>(), ms in 0_u64..10_000) {
        let (first, _) = two_rates(a, b, ms);
        prop_assert_eq!(first, Reading::Warming);
    }

    #[test]
    fn a_counter_going_back_never_reads_a_value(a in 1_u64.., ms in any::<u64>()) {
        let (_, second) = two_rates(a, a.saturating_sub(1), ms);
        prop_assert_eq!(second, Reading::Warming);
    }

    #[test]
    fn a_growing_counter_always_reads_a_value(a in any::<u64>(), d in any::<u64>(), ms in 1_u64..1_000_000) {
        let (_, second) = two_rates(a, a.saturating_add(d), ms);
        prop_assert!(second.measure().is_some(), "{second:?}");
    }
}
