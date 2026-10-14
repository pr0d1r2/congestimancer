use std::time::Instant;

use super::Source;
use crate::signal::{FakeSource, Measure, Reading, SignalId};
use crate::units::Count;

/// A reading of `n` things.
fn count(n: u64) -> Reading {
    Reading::Value(Measure::Count(Count(n)))
}

#[test]
fn a_fake_gives_its_script_then_repeats_the_last() {
    let mut fake = FakeSource::new();
    fake.script(SignalId::IoQueue, [Reading::Warming, count(1), count(2)]);
    let now = Instant::now();
    let got: Vec<Reading> =
        (0..4).map(|_| fake.read(SignalId::IoQueue, now)).collect();
    assert_eq!(got, [Reading::Warming, count(1), count(2), count(2)]);
}

#[test]
fn an_unscripted_signal_is_unsupported() {
    let mut fake = FakeSource::new();
    let r = fake.read(SignalId::Gpu, Instant::now());
    assert!(matches!(r, Reading::Unsupported { .. }), "{r:?}");
}

#[test]
fn a_fake_counts_reads_per_signal() {
    let mut fake = FakeSource::new();
    fake.script(SignalId::Load, [count(1)]);
    let now = Instant::now();
    let _ = fake.read(SignalId::Load, now);
    let _ = read_via(&mut fake, SignalId::Load);
    assert_eq!(fake.reads(SignalId::Load), 2);
    assert_eq!(fake.reads(SignalId::Mem), 0);
}

/// Reads `id` through any source, here a borrowed one.
fn read_via<S: Source>(mut source: S, id: SignalId) -> Reading {
    source.read(id, Instant::now())
}

#[test]
fn a_boxed_source_reads_through() {
    let mut fake = FakeSource::new();
    fake.script(SignalId::Load, [count(3)]);
    let mut boxed: Box<dyn Source> = Box::new(fake);
    assert_eq!(boxed.read(SignalId::Load, Instant::now()), count(3));
}
