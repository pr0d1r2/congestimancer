use std::time::Instant;

use super::{SignalSet, refresh};
use crate::signal::{FakeSource, Measure, Reading, SignalId};
use crate::units::Count;

#[test]
fn a_set_holds_what_was_inserted() {
    let mut set = SignalSet::new();
    assert!(set.is_empty());
    assert!(set.insert(SignalId::Gpu));
    assert!(!set.insert(SignalId::Gpu));
    assert!(set.contains(SignalId::Gpu));
    assert!(!set.contains(SignalId::Load));
    assert_eq!(set.len(), 1);
}

#[test]
fn every_signal_fits_in_a_set() {
    let all: SignalSet = SignalId::ALL.into_iter().collect();
    assert_eq!(all.len(), SignalId::ALL.len());
    assert!(all.iter().eq(SignalId::ALL));
}

#[test]
fn switches_map_to_the_signals_they_read() {
    let switches = ["io-busy", "load", "all-mem", "needs-mem", "interval"];
    let set = SignalSet::from_switches(switches);
    let got: Vec<SignalId> = set.iter().collect();
    assert_eq!(got, [SignalId::Load, SignalId::Mem, SignalId::IoBusy]);
}

#[test]
fn a_set_with_a_rate_needs_two_samples() {
    let gauges = SignalSet::from_switches(["load", "mem"]);
    assert!(!gauges.needs_two_samples());
    let rates = SignalSet::from_switches(["load", "net-rx"]);
    assert!(rates.needs_two_samples());
}

#[test]
fn refresh_touches_only_the_wanted_signals() {
    let one = Reading::Value(Measure::Count(Count(1)));
    let mut fake = FakeSource::new();
    for id in [SignalId::Load, SignalId::Mem, SignalId::Gpu] {
        fake.script(id, [one.clone()]);
    }
    let wanted = SignalSet::from_switches(["load"]);
    let got = refresh(&mut fake, wanted, Instant::now());
    assert_eq!(got, [(SignalId::Load, one)]);
    assert_eq!(fake.reads(SignalId::Load), 1);
    assert_eq!(fake.reads(SignalId::Mem), 0);
    assert_eq!(fake.reads(SignalId::Gpu), 0);
}
