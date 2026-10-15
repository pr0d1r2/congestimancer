use std::time::Duration;

use super::{Admit, Open, Outcome, Rig, poll};
use crate::gate::{Gate, Gates};
use crate::signal::{FakeSource, ManualClock, Measure, Reading, SignalId};
use crate::units::Decimal;
use crate::wait::{MIN_INTERVAL, Pacing};

const SEC: Duration = Duration::from_secs(1);

fn secs(n: u64) -> Duration {
    Duration::from_secs(n)
}

fn load(thousandths: u64) -> Reading {
    Reading::Value(Measure::Number(Decimal::from_thousandths(thousandths)))
}

/// Load at 1.0 passes `--load 2`; at 3.0 it fails.
const PASS: u64 = 1_000;
const FAIL: u64 = 3_000;

fn load_gate() -> Gates {
    let mut gates = Gates::new();
    let two = Measure::Number(Decimal::from_thousandths(2_000));
    gates.push(Gate::below(SignalId::Load, two));
    gates
}

/// One wait's parts, and what it wrote and slept.
struct Bench {
    gates: Gates,
    source: FakeSource,
    clock: ManualClock,
    log: Vec<u8>,
}

impl Bench {
    /// Load reads `script` in order, then its last value forever.
    fn new(script: &[u64]) -> Self {
        let mut source = FakeSource::new();
        source.script(SignalId::Load, script.iter().map(|&t| load(t)));
        Self {
            gates: load_gate(),
            source,
            clock: ManualClock::new(),
            log: Vec::new(),
        }
    }

    fn run<A: Admit>(
        &mut self,
        pacing: &Pacing,
        admit: &mut A,
    ) -> Result<Outcome<A::Held>, A::Error> {
        let rig = Rig::new(
            &mut self.gates,
            &mut self.source,
            &mut self.clock,
            &mut self.log,
        );
        poll(rig, pacing, admit)
    }

    fn open(&mut self, pacing: &Pacing) -> Outcome<()> {
        match self.run(pacing, &mut Open) {
            Ok(outcome) => outcome,
            Err(never) => match never {},
        }
    }

    fn sleeps(&self) -> Vec<Duration> {
        self.clock.sleeps().to_vec()
    }

    fn rounds(&self) -> usize {
        self.source.reads(SignalId::Load)
    }
}

fn released(outcome: &Outcome<()>) -> bool {
    matches!(outcome, Outcome::Released(()))
}

#[test]
fn passing_gates_release_at_once_without_sleeping() {
    let mut bench = Bench::new(&[PASS]);
    assert!(released(&bench.open(&Pacing::new(secs(2)))));
    assert!(bench.sleeps().is_empty());
    assert_eq!(bench.rounds(), 1);
}

#[test]
fn a_failing_round_sleeps_one_interval_then_checks_again() {
    let mut bench = Bench::new(&[FAIL, FAIL, PASS]);
    assert!(released(&bench.open(&Pacing::new(secs(2)))));
    assert_eq!(bench.sleeps(), [secs(2), secs(2)]);
    assert_eq!(bench.rounds(), 3);
}

#[test]
fn the_default_interval_is_two_seconds() {
    let mut bench = Bench::new(&[FAIL, PASS]);
    assert!(released(&bench.open(&Pacing::default())));
    assert_eq!(bench.sleeps(), [secs(2)]);
}

#[test]
fn a_zero_interval_still_sleeps_never_spins() {
    let mut bench = Bench::new(&[FAIL, FAIL, PASS]);
    assert!(released(&bench.open(&Pacing::new(Duration::ZERO))));
    assert_eq!(bench.sleeps(), [MIN_INTERVAL, MIN_INTERVAL]);
}

#[test]
fn warming_waits_like_a_failure() {
    let mut source = FakeSource::new();
    source.script(SignalId::Load, [Reading::Warming, load(PASS)]);
    let mut bench = Bench::new(&[]);
    bench.source = source;
    assert!(released(&bench.open(&Pacing::new(SEC))));
    assert_eq!(bench.sleeps(), [SEC]);
}

#[test]
fn exit_codes_follow_the_outcome() {
    assert_eq!(Outcome::Released(()).exit_code(), None);
    assert_eq!(Outcome::<()>::TimedOut(None).exit_code(), Some(124));
}
