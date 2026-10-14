use std::time::Duration;

use super::{Sample, Sampled, Signal};
use crate::signal::{
    Clock, Counter, ManualClock, Measure, Reading, SignalId, Source,
};
use crate::units::{Bytes, Rate};

/// A domain that gives the next of its samples on each call.
struct Scripted(Vec<Sample>);

impl Signal for Scripted {
    fn sample(&mut self, _: SignalId) -> Sample {
        self.0.pop().unwrap_or(Sample::Unsupported("dry".into()))
    }
}

/// Reads `samples` (first one first) one second apart.
fn read(mut samples: Vec<Sample>) -> Vec<Reading> {
    let n = samples.len();
    samples.reverse();
    let mut source = Sampled::new(Scripted(samples));
    let mut clock = ManualClock::new();
    (0..n)
        .map(|_| {
            let r = source.read(SignalId::NetRx, clock.now());
            clock.advance(Duration::from_secs(1));
            r
        })
        .collect()
}

const fn total(total: u64) -> Sample {
    Sample::Counter(Counter::Rate { total })
}

#[test]
fn a_gauge_passes_through() {
    let m = Measure::Bytes(Bytes(42));
    assert_eq!(read(vec![Sample::Gauge(m)]), [Reading::Value(m)]);
}

#[test]
fn a_counter_goes_through_the_sampler() {
    let got = read(vec![total(0), total(64)]);
    let rate = Reading::Value(Measure::Rate(Rate(64)));
    assert_eq!(got, [Reading::Warming, rate]);
}

#[test]
fn unsupported_reads_unsupported_and_drops_the_base() {
    let gone = Sample::Unsupported("interface gone".into());
    let got = read(vec![total(0), gone, total(64)]);
    let want = [
        Reading::Warming,
        Reading::unsupported("interface gone"),
        Reading::Warming,
    ];
    assert_eq!(got, want);
}

#[test]
fn the_wrapped_signal_stays_reachable() {
    let mut source = Sampled::new(Scripted(vec![]));
    source.get_mut().0.push(total(1));
    assert_eq!(source.get_ref().0.len(), 1);
}
