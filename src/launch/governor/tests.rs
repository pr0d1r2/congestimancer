use std::collections::VecDeque;
use std::io;
use std::os::unix::process::ExitStatusExt as _;
use std::process::ExitStatus;
use std::time::{Duration, Instant};

use super::{GovernRig, Group, Judge, LocalGates, Mates, govern};
use crate::gate::{Gate, Gates, Overall};
use crate::launch::GroupAction;
use crate::signal::{FakeSource, ManualClock, Measure, Reading, SignalId};
use crate::sys::{ForwardSignal, GroupSignal, Wake};
use crate::units::Decimal;

const INTERVAL: Duration = Duration::from_secs(2);

/// Rounds that say `script` in order, then its last forever.
struct Script(VecDeque<Overall>);

impl Script {
    fn new(script: &[Overall]) -> Self {
        Self(script.iter().copied().collect())
    }
}

impl Judge for Script {
    fn judge(&mut self, _: Instant) -> Overall {
        match self.0.len() {
            0 => Overall::Pass,
            1 => self.0.front().copied().unwrap_or(Overall::Pass),
            _ => self.0.pop_front().unwrap_or(Overall::Pass),
        }
    }
}

/// A group whose waits return `wakes` in order (then time out) and whose
/// command ends with `status` after `exit_after` waits.
struct FakeGroup {
    wakes: VecDeque<Option<Wake>>,
    exit_after: usize,
    status: ExitStatus,
    waits: Vec<Duration>,
    sent: Vec<GroupSignal>,
    refuse: bool,
}

impl FakeGroup {
    fn new(exit_after: usize, code: i32) -> Self {
        Self {
            wakes: VecDeque::new(),
            exit_after,
            status: ExitStatus::from_raw(code << 8),
            waits: Vec::new(),
            sent: Vec::new(),
            refuse: false,
        }
    }

    fn wakes(mut self, wakes: &[Option<Wake>]) -> Self {
        self.wakes = wakes.iter().copied().collect();
        self
    }
}

impl Group for FakeGroup {
    fn signal(&mut self, sig: GroupSignal) -> io::Result<()> {
        self.sent.push(sig);
        if self.refuse {
            return Err(io::Error::other("no such group"));
        }
        Ok(())
    }

    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        Ok((self.waits.len() >= self.exit_after).then_some(self.status))
    }

    fn wait(&mut self, d: Duration) -> io::Result<Option<Wake>> {
        assert!(self.waits.len() < 1_000, "the command never ended");
        self.waits.push(d);
        Ok(self.wakes.pop_front().flatten())
    }
}

fn run(judge: &mut dyn Judge, group: &mut FakeGroup) -> (u8, String) {
    let clock = ManualClock::new();
    let mut log = Vec::new();
    let rig = GovernRig::new(judge, &clock, group, &mut log);
    let code = govern(rig, INTERVAL);
    let code = code.unwrap_or_else(|e| unreachable!("govern failed: {e}"));
    (code, String::from_utf8_lossy(&log).into_owned())
}

#[test]
fn passing_gates_leave_the_command_alone_and_pass_its_code_on() {
    let mut group = FakeGroup::new(3, 3);
    let (code, _) = run(&mut Script::new(&[Overall::Pass]), &mut group);
    assert_eq!(code, 3);
    assert!(group.sent.is_empty(), "{:?}", group.sent);
    assert_eq!(group.waits, vec![INTERVAL; 3]);
}

#[test]
fn a_failing_round_stops_the_group_and_a_pass_continues_it() {
    let rounds = [Overall::Fail, Overall::Fail, Overall::Pass];
    let mut group = FakeGroup::new(4, 0);
    let (code, log) = run(&mut Script::new(&rounds), &mut group);
    assert_eq!(code, 0);
    assert_eq!(group.sent, vec![GroupSignal::Stop, GroupSignal::Cont]);
    assert!(log.contains("paused"), "{log}");
    assert!(log.contains("resumed"), "{log}");
}

#[test]
fn warming_or_unsupported_rounds_change_nothing() {
    let rounds = [Overall::Warming, Overall::Unsupported];
    let mut group = FakeGroup::new(3, 0);
    run(&mut Script::new(&rounds), &mut group);
    assert!(group.sent.is_empty(), "{:?}", group.sent);
}

#[test]
fn a_caught_signal_is_forwarded_to_the_group() {
    let term = Some(Wake::Forward(ForwardSignal::Term));
    let mut group = FakeGroup::new(2, 0).wakes(&[term]);
    run(&mut Script::new(&[Overall::Pass]), &mut group);
    assert_eq!(group.sent, vec![GroupSignal::Term]);
}

#[test]
fn a_signal_to_a_stopped_group_continues_it_so_it_can_act() {
    let hup = Some(Wake::Forward(ForwardSignal::Hup));
    let mut group = FakeGroup::new(2, 0).wakes(&[None, hup]);
    run(
        &mut Script::new(&[Overall::Fail, Overall::Fail]),
        &mut group,
    );
    let want = [GroupSignal::Stop, GroupSignal::Hup, GroupSignal::Cont];
    assert_eq!(group.sent, want);
}

#[test]
fn a_signal_or_child_wake_is_no_round() {
    let int = Some(Wake::Forward(ForwardSignal::Int));
    let child = Some(Wake::Child);
    let mut group = FakeGroup::new(2, 0).wakes(&[int, child]);
    run(&mut Script::new(&[Overall::Fail]), &mut group);
    assert_eq!(group.sent, vec![GroupSignal::Int]);
}

#[test]
fn a_command_that_ends_while_stopped_leaves_its_group_continued() {
    let mut group = FakeGroup::new(1, 5);
    let (code, _) = run(&mut Script::new(&[Overall::Fail]), &mut group);
    assert_eq!(code, 5);
    assert_eq!(group.sent, vec![GroupSignal::Stop, GroupSignal::Cont]);
}

#[test]
fn death_by_signal_exits_128_plus_n() {
    let mut group = FakeGroup::new(0, 0);
    group.status = ExitStatus::from_raw(libc::SIGTERM);
    let (code, _) = run(&mut Script::new(&[Overall::Pass]), &mut group);
    assert_eq!(code, 143);
}

#[test]
fn a_refused_signal_is_logged_and_the_loop_goes_on() {
    let mut group = FakeGroup::new(2, 0);
    group.refuse = true;
    let (code, log) = run(&mut Script::new(&[Overall::Fail]), &mut group);
    assert_eq!(code, 0);
    assert!(log.contains("no such group"), "{log}");
}

fn load(thousandths: u64) -> Reading {
    Reading::Value(Measure::Number(Decimal::from_thousandths(thousandths)))
}

#[test]
fn local_gates_judge_each_round_from_the_source() {
    let mut gates = Gates::new();
    let two = Measure::Number(Decimal::from_thousandths(2_000));
    gates.push(Gate::below(SignalId::Load, two));
    let mut source = FakeSource::new();
    source.script(SignalId::Load, [load(3_000), load(1_000)]);
    let mut judge = LocalGates::new(&mut gates, &mut source);
    let now = Instant::now();
    assert_eq!(judge.judge(now), Overall::Fail);
    assert_eq!(judge.judge(now), Overall::Pass);
    drop(judge);
    assert_eq!(source.reads(SignalId::Load), 2);
}

/// A crew that does what `scripted` says, noting what it saw.
struct FakeMates {
    reply: Option<GroupAction>,
    calms: Vec<Duration>,
    resumed: usize,
    broken: bool,
    done: Vec<bool>,
}

impl FakeMates {
    fn new(reply: Option<GroupAction>) -> Self {
        Self {
            reply,
            calms: Vec::new(),
            resumed: 0,
            broken: false,
            done: Vec::new(),
        }
    }
}

impl Mates for FakeMates {
    fn round(
        &mut self,
        _: Overall,
        calm: Duration,
        act: &mut dyn FnMut(GroupAction) -> bool,
    ) -> io::Result<()> {
        self.calms.push(calm);
        if self.broken {
            return Err(io::Error::other("crew lost"));
        }
        if let Some(action) = self.reply {
            self.done.push(act(action));
        }
        Ok(())
    }

    fn resumed(&mut self) {
        self.resumed = self.resumed.saturating_add(1);
    }
}

fn crewed(
    judge: &mut dyn Judge,
    group: &mut FakeGroup,
    mates: &mut FakeMates,
) -> (u8, String) {
    let clock = ManualClock::new();
    let mut log = Vec::new();
    let rig = GovernRig::new(judge, &clock, group, &mut log);
    let code = govern(rig.with_mates(mates), INTERVAL);
    let code = code.unwrap_or_else(|e| unreachable!("govern failed: {e}"));
    (code, String::from_utf8_lossy(&log).into_owned())
}

#[test]
fn with_a_crew_its_turn_decides_not_the_gates_alone() {
    let mut group = FakeGroup::new(2, 0);
    let mut mates = FakeMates::new(None);
    crewed(&mut Script::new(&[Overall::Fail]), &mut group, &mut mates);
    assert!(group.sent.is_empty(), "{:?}", group.sent);
    assert_eq!(mates.calms.len(), 2);
}

#[test]
fn a_crew_that_says_stop_stops_the_group_and_hears_it_was_done() {
    let mut group = FakeGroup::new(1, 0);
    let mut mates = FakeMates::new(Some(GroupAction::Stop));
    crewed(&mut Script::new(&[Overall::Fail]), &mut group, &mut mates);
    assert_eq!(group.sent, vec![GroupSignal::Stop, GroupSignal::Cont]);
    assert_eq!(mates.done, vec![true]);
    assert_eq!(mates.resumed, 1);
}

#[test]
fn a_refused_stop_is_reported_as_not_done() {
    let mut group = FakeGroup::new(1, 0);
    group.refuse = true;
    let mut mates = FakeMates::new(Some(GroupAction::Stop));
    crewed(&mut Script::new(&[Overall::Fail]), &mut group, &mut mates);
    assert_eq!(mates.done, vec![false]);
}

#[test]
fn the_calm_time_counts_passing_rounds_and_breaks_on_a_failure() {
    let mut group = FakeGroup::new(4, 0);
    let mut mates = FakeMates::new(None);
    let script = [Overall::Pass, Overall::Pass, Overall::Fail, Overall::Pass];
    crewed(&mut Script::new(&script), &mut group, &mut mates);
    let two = INTERVAL * 2;
    assert_eq!(mates.calms.len(), 4);
    // ManualClock stands still, so a streak reads zero but never carries
    // over a failure.
    assert!(mates.calms.iter().all(|c| *c <= two), "{:?}", mates.calms);
}

#[test]
fn a_crew_that_cannot_be_read_is_logged_and_the_loop_goes_on() {
    let mut group = FakeGroup::new(2, 4);
    let mut mates = FakeMates::new(None);
    mates.broken = true;
    let (code, log) =
        crewed(&mut Script::new(&[Overall::Fail]), &mut group, &mut mates);
    assert_eq!(code, 4);
    assert!(log.contains("crew lost"), "{log}");
}
