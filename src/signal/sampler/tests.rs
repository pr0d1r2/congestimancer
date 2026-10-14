use std::time::Duration;

use super::{Counter, Sampler};
use crate::signal::{Clock, ManualClock, Measure, Reading, SignalId};
use crate::units::{Percent, Rate};

/// Feeds `counters` to one signal, `step` apart, and collects the readings.
fn run(counters: &[Counter], step: Duration) -> Vec<Reading> {
    let mut clock = ManualClock::new();
    let mut sampler = Sampler::new();
    counters
        .iter()
        .map(|&c| {
            let r = sampler.update(SignalId::Io, c, clock.now());
            clock.advance(step);
            r
        })
        .collect()
}

/// A percent reading.
fn percent(hundredths: u16) -> Reading {
    let p = Percent::from_hundredths(hundredths).unwrap_or_default();
    Reading::Value(Measure::Percent(p))
}

/// A rate reading.
fn rate(bytes_per_second: u64) -> Reading {
    Reading::Value(Measure::Rate(Rate(bytes_per_second)))
}

/// A duration reading.
fn mean(ms: u64) -> Reading {
    Reading::Value(Measure::Duration(Duration::from_millis(ms)))
}

const SECOND: Duration = Duration::from_secs(1);

const fn total(total: u64) -> Counter {
    Counter::Rate { total }
}

const fn share(part: u64, whole: u64) -> Counter {
    Counter::Share { part, whole }
}

const fn busy(ms: u64) -> Counter {
    Counter::Busy {
        busy: Duration::from_millis(ms),
    }
}

const fn events(ms: u64, events: u64) -> Counter {
    Counter::Mean {
        time: Duration::from_millis(ms),
        events,
    }
}

#[test]
fn the_first_sample_warms_up_for_every_kind() {
    for c in [total(5), share(1, 2), busy(3), events(4, 5)] {
        assert_eq!(run(&[c], SECOND), [Reading::Warming], "{c:?}");
    }
}

#[test]
fn a_rate_is_bytes_per_second_over_the_interval() {
    let got = run(&[total(0), total(1000)], Duration::from_secs(2));
    assert_eq!(got, [Reading::Warming, rate(500)]);
}

#[test]
fn a_share_is_the_part_of_the_whole() {
    let got = run(&[share(10, 100), share(30, 200)], SECOND);
    assert_eq!(got, [Reading::Warming, percent(2000)]);
    let over = run(&[share(0, 0), share(9, 3)], SECOND);
    assert_eq!(over, [Reading::Warming, percent(10_000)]);
}

#[test]
fn busy_is_the_part_of_the_interval_spent_busy() {
    let got = run(&[busy(0), busy(500)], SECOND);
    assert_eq!(got, [Reading::Warming, percent(5000)]);
}

#[test]
fn a_mean_is_time_per_event_and_zero_without_events() {
    let got = run(&[events(0, 0), events(30, 3), events(30, 3)], SECOND);
    assert_eq!(got, [Reading::Warming, mean(10), mean(0)]);
}

#[test]
fn a_counter_going_back_warms_up_again_from_the_new_base() {
    let got = run(&[total(900), total(100), total(400)], SECOND);
    assert_eq!(got, [Reading::Warming, Reading::Warming, rate(300)]);
    let shares = run(&[share(5, 10), share(6, 4)], SECOND);
    assert_eq!(shares, [Reading::Warming, Reading::Warming]);
}

#[test]
fn a_change_of_kind_warms_up_again() {
    let got = run(&[total(0), busy(0), busy(100)], SECOND);
    assert_eq!(got, [Reading::Warming, Reading::Warming, percent(1000)]);
}

#[test]
fn no_time_or_no_whole_keeps_the_base() {
    let got = run(&[total(0), total(10), total(30)], Duration::ZERO);
    assert_eq!(got, [const { Reading::Warming }; 3]);
    let still = run(&[share(0, 10), share(0, 10), share(5, 20)], SECOND);
    assert_eq!(still, [Reading::Warming, Reading::Warming, percent(5000)]);
}

#[test]
fn signals_keep_their_own_base_and_forget_drops_it() {
    let mut clock = ManualClock::new();
    let mut s = Sampler::new();
    let _ = s.update(SignalId::NetRx, total(0), clock.now());
    let _ = s.update(SignalId::NetTx, total(0), clock.now());
    clock.advance(SECOND);
    assert_eq!(s.update(SignalId::NetRx, total(7), clock.now()), rate(7));
    s.forget(SignalId::NetTx);
    let tx = s.update(SignalId::NetTx, total(9), clock.now());
    assert_eq!(tx, Reading::Warming);
}
