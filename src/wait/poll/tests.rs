use std::time::{Duration, Instant};

use super::{Admit, Open, Outcome, Rig, poll};
use crate::gate::{Gate, Gates, Overall};
use crate::report::Style;
use crate::signal::{
    Clock, FakeSource, ManualClock, Measure, Reading, SignalId,
};
use crate::units::Decimal;
use crate::wait::{MIN_INTERVAL, Pacing, jitter_delay};

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

/// A [`ManualClock`] that fails the test instead of letting a wait that
/// never ends hang it.
struct Bounded(ManualClock);

impl Clock for Bounded {
    fn now(&self) -> Instant {
        self.0.now()
    }

    fn sleep(&mut self, d: Duration) {
        assert!(self.0.sleeps().len() < 1_000, "the wait never ended");
        self.0.sleep(d);
    }
}

/// One wait's parts, and what it wrote and slept.
struct Bench {
    gates: Gates,
    source: FakeSource,
    clock: Bounded,
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
            clock: Bounded(ManualClock::new()),
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
        self.clock.0.sleeps().to_vec()
    }

    fn log(&self) -> String {
        String::from_utf8_lossy(&self.log).into_owned()
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

/// Slot steps that say wait `left` times, then let the command go;
/// counts how often they were tried.
#[derive(Default)]
struct Turns {
    left: usize,
    tries: usize,
}

impl Admit for Turns {
    type Held = ();
    type Error = std::convert::Infallible;

    fn try_admit(&mut self) -> Result<Option<()>, Self::Error> {
        self.tries = self.tries.saturating_add(1);
        if self.left == 0 {
            return Ok(Some(()));
        }
        self.left = self.left.saturating_sub(1);
        Ok(None)
    }
}

fn backoff(interval: u64, cap: u64) -> Pacing {
    let mut pacing = Pacing::new(secs(interval));
    pacing.backoff = Some(secs(cap));
    pacing
}

#[test]
fn backoff_doubles_each_consecutive_failure_up_to_the_cap() {
    let mut bench = Bench::new(&[FAIL, FAIL, FAIL, FAIL, FAIL, PASS]);
    assert!(released(&bench.open(&backoff(1, 5))));
    assert_eq!(
        bench.sleeps(),
        [secs(1), secs(2), secs(4), secs(5), secs(5)]
    );
}

#[test]
fn a_passing_round_resets_backoff_to_the_interval() {
    let mut bench = Bench::new(&[FAIL, FAIL, FAIL, PASS, FAIL, PASS]);
    let mut turns = Turns {
        left: 1,
        ..Turns::default()
    };
    assert!(bench.run(&backoff(1, 60), &mut turns).is_ok());
    assert_eq!(
        bench.sleeps(),
        [secs(1), secs(2), secs(4), secs(1), secs(1)]
    );
}

#[test]
fn warming_neither_backs_off_nor_resets() {
    let mut source = FakeSource::new();
    let warm = Reading::Warming;
    source.script(
        SignalId::Load,
        [load(FAIL), load(FAIL), warm.clone(), warm, load(PASS)],
    );
    let mut bench = Bench::new(&[]);
    bench.source = source;
    assert!(released(&bench.open(&backoff(1, 60))));
    assert_eq!(bench.sleeps(), [secs(1), secs(2), secs(2), secs(2)]);
}

#[test]
fn a_cap_below_the_interval_never_shortens_it() {
    let mut bench = Bench::new(&[FAIL, FAIL, PASS]);
    assert!(released(&bench.open(&backoff(4, 1))));
    assert_eq!(bench.sleeps(), [secs(4), secs(4)]);
}

#[test]
fn without_backoff_the_interval_stays_put() {
    let mut bench = Bench::new(&[FAIL, FAIL, FAIL, PASS]);
    assert!(released(&bench.open(&Pacing::new(SEC))));
    assert_eq!(bench.sleeps(), [SEC, SEC, SEC]);
}

fn timeout(interval: u64, limit: u64) -> Pacing {
    let mut pacing = Pacing::new(secs(interval));
    pacing.timeout = Some(secs(limit));
    pacing
}

fn timed_out_failing(outcome: &Outcome<()>) -> bool {
    matches!(
        outcome,
        Outcome::TimedOut(Some(last)) if last.overall == Overall::Fail
    )
}

#[test]
fn timeout_gives_up_after_a_last_round_at_the_deadline() {
    let mut bench = Bench::new(&[FAIL]);
    let outcome = bench.open(&timeout(2, 5));
    assert!(timed_out_failing(&outcome));
    assert_eq!(outcome.exit_code(), Some(124));
    assert_eq!(bench.sleeps(), [secs(2), secs(2), secs(1)]);
    assert_eq!(bench.rounds(), 4);
}

#[test]
fn a_zero_timeout_checks_once() {
    let mut bench = Bench::new(&[FAIL]);
    assert!(timed_out_failing(&bench.open(&timeout(2, 0))));
    assert!(bench.sleeps().is_empty());
    assert_eq!(bench.rounds(), 1);
}

#[test]
fn a_pass_before_the_deadline_releases() {
    let mut bench = Bench::new(&[FAIL, FAIL, PASS]);
    assert!(released(&bench.open(&timeout(2, 5))));
}

#[test]
fn a_pass_on_the_deadline_still_releases() {
    let mut bench = Bench::new(&[FAIL, FAIL, FAIL, PASS]);
    assert!(released(&bench.open(&timeout(2, 5))));
    assert_eq!(bench.rounds(), 4);
}

fn settle(interval: u64, hold: u64) -> Pacing {
    let mut pacing = Pacing::new(secs(interval));
    pacing.settle = Some(secs(hold));
    pacing
}

#[test]
fn settle_holds_a_pass_for_its_whole_span_before_release() {
    let mut bench = Bench::new(&[PASS]);
    let mut turns = Turns::default();
    assert!(bench.run(&settle(2, 3), &mut turns).is_ok());
    assert_eq!(bench.sleeps(), [secs(2), secs(1)]);
    assert_eq!(bench.rounds(), 3);
    assert_eq!(turns.tries, 1);
}

#[test]
fn a_break_in_the_pass_restarts_settle() {
    let mut bench = Bench::new(&[PASS, FAIL, PASS]);
    assert!(released(&bench.open(&settle(2, 3))));
    assert_eq!(bench.sleeps(), [secs(2), secs(2), secs(2), secs(1)]);
    assert_eq!(bench.rounds(), 5);
}

#[test]
fn warming_breaks_settle_too() {
    let mut source = FakeSource::new();
    let script = [load(PASS), Reading::Warming, load(PASS)];
    source.script(SignalId::Load, script);
    let mut bench = Bench::new(&[]);
    bench.source = source;
    assert!(released(&bench.open(&settle(2, 3))));
    assert_eq!(bench.rounds(), 5);
}

#[test]
fn an_unsupported_gate_ends_the_wait_at_once() {
    let mut bench = Bench::new(&[]);
    let mut turns = Turns::default();
    let outcome = bench.run(&settle(2, 3), &mut turns);
    assert!(matches!(&outcome, Ok(Outcome::Unsupported(_))));
    assert_eq!(outcome.ok().and_then(|o| o.exit_code()), Some(69));
    assert!(bench.sleeps().is_empty());
    assert_eq!(turns.tries, 0);
}

#[test]
fn a_gate_that_turns_unsupported_mid_wait_ends_it() {
    let mut source = FakeSource::new();
    let gone = Reading::unsupported("sensor gone");
    source.script(SignalId::Load, [load(FAIL), gone]);
    let mut bench = Bench::new(&[]);
    bench.source = source;
    let outcome = bench.open(&timeout(2, 60));
    assert!(matches!(
        outcome,
        Outcome::Unsupported(e) if e.overall == Overall::Unsupported
    ));
    assert_eq!(bench.sleeps(), [secs(2)]);
}

fn jitter(seed: u64, span: u64) -> Pacing {
    let mut pacing = Pacing::new(SEC);
    pacing.jitter = Some(secs(span));
    pacing.seed = seed;
    pacing
}

#[test]
fn jitter_sleeps_its_seeded_pick_after_release() {
    let mut bench = Bench::new(&[FAIL, PASS]);
    assert!(released(&bench.open(&jitter(5, 10))));
    let pick = jitter_delay(5, secs(10));
    assert_eq!(bench.sleeps(), [SEC, pick]);
}

#[test]
fn a_zero_pick_sleeps_nothing() {
    let mut bench = Bench::new(&[PASS]);
    assert!(released(&bench.open(&jitter(5, 0))));
    assert!(bench.sleeps().is_empty());
}

#[test]
fn no_jitter_after_a_timeout() {
    let mut bench = Bench::new(&[FAIL]);
    let mut pacing = jitter(5, 10);
    pacing.timeout = Some(Duration::ZERO);
    assert!(matches!(bench.open(&pacing), Outcome::TimedOut(_)));
    assert!(bench.sleeps().is_empty());
}

fn heartbeat(every: u64, limit: u64) -> Pacing {
    let mut pacing = timeout(2, limit);
    pacing.heartbeat = Some(secs(every));
    pacing
}

#[test]
fn heartbeat_says_why_it_waits_every_span() {
    let mut bench = Bench::new(&[FAIL]);
    assert!(timed_out_failing(&bench.open(&heartbeat(5, 11))));
    let log = bench.log();
    let waited: Vec<&str> = log
        .lines()
        .filter_map(|l| l.strip_prefix("cgmr: waiting "))
        .filter_map(|l| l.split(':').next())
        .collect();
    assert_eq!(waited, ["5s", "10s"], "{log}");
    assert!(log.lines().all(|l| l.contains("--load")), "{log}");
}

#[test]
fn heartbeat_wakes_add_no_rounds() {
    let mut bench = Bench::new(&[FAIL]);
    assert!(timed_out_failing(&bench.open(&heartbeat(5, 11))));
    let two = secs(2);
    assert_eq!(bench.sleeps(), [two, two, SEC, SEC, two, two, SEC]);
    assert_eq!(bench.rounds(), 7);
}

#[test]
fn no_heartbeat_when_released_before_the_first() {
    let mut bench = Bench::new(&[FAIL, PASS]);
    assert!(released(&bench.open(&heartbeat(5, 60))));
    assert_eq!(bench.log(), "");
}

#[test]
fn verbose_names_the_blocking_gate_each_failing_round() {
    let mut bench = Bench::new(&[FAIL, FAIL, PASS]);
    let mut pacing = Pacing::new(SEC);
    pacing.verbose = Some(Style::Plain);
    assert!(released(&bench.open(&pacing)));
    let log = bench.log();
    assert_eq!(log.lines().count(), 2, "{log}");
    assert!(
        log.lines().all(|l| l.starts_with("blocked: --load")),
        "{log}"
    );
}

#[test]
fn quiet_by_default() {
    let mut bench = Bench::new(&[FAIL, FAIL, PASS]);
    assert!(released(&bench.open(&Pacing::new(SEC))));
    assert_eq!(bench.log(), "");
}
