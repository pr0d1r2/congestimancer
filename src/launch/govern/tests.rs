use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use super::{GroupAction, govern_step, status_code};
use crate::gate::Overall;

#[test]
fn a_failing_gate_stops_a_running_group() {
    assert_eq!(govern_step(false, Overall::Fail), Some(GroupAction::Stop));
    assert_eq!(govern_step(true, Overall::Fail), None);
}

#[test]
fn passing_again_continues_a_stopped_group() {
    let resume = Some(GroupAction::Continue);
    assert_eq!(govern_step(true, Overall::Pass), resume);
    assert_eq!(govern_step(false, Overall::Pass), None);
}

#[test]
fn warming_or_unsupported_leaves_the_group_as_it_is() {
    for stopped in [false, true] {
        assert_eq!(govern_step(stopped, Overall::Warming), None);
        assert_eq!(govern_step(stopped, Overall::Unsupported), None);
    }
}

#[test]
fn the_command_exit_code_passes_on() {
    assert_eq!(status_code(ExitStatus::from_raw(0)), 0);
    assert_eq!(status_code(ExitStatus::from_raw(3 << 8)), 3);
    assert_eq!(status_code(ExitStatus::from_raw(255 << 8)), 255);
}

#[test]
fn death_by_signal_is_128_plus_n() {
    assert_eq!(status_code(ExitStatus::from_raw(2)), 130);
    assert_eq!(status_code(ExitStatus::from_raw(15)), 143);
    assert_eq!(status_code(ExitStatus::from_raw(9)), 137);
}

mod crew {
    use super::{GroupAction, Overall};
    use crate::launch::{Turn, crew_step};
    use crate::slot::Member;

    const SEC: u64 = 1_000_000_000;

    fn sec(n: u64) -> u64 {
        n.saturating_mul(SEC)
    }

    fn member(name: &str, started: u64, stopped: Option<u64>) -> Member {
        let moved = stopped.unwrap_or(0);
        Member {
            name: name.to_owned(),
            started: sec(started),
            stopped: stopped.map(sec),
            moved: sec(moved),
        }
    }

    fn step(
        members: &[Member],
        me: &str,
        overall: Overall,
        keep: u64,
    ) -> Option<GroupAction> {
        let turn = Turn {
            members,
            me,
            overall,
            calm: true,
            now: 100 * SEC,
            interval: 5 * SEC,
            settle: 30 * SEC,
            keep,
        };
        crew_step(&turn)
    }

    #[test]
    fn only_the_youngest_running_job_stops() {
        let all = [member("a", 1, None), member("b", 2, None)];
        let stop = Some(GroupAction::Stop);
        assert_eq!(step(&all, "b", Overall::Fail, 1), stop);
        assert_eq!(step(&all, "a", Overall::Fail, 1), None);
    }

    #[test]
    fn the_last_running_job_is_kept() {
        let all = [member("a", 1, Some(10)), member("b", 2, None)];
        assert_eq!(step(&all, "b", Overall::Fail, 1), None);
        let two = [member("a", 1, None), member("b", 2, None)];
        assert_eq!(step(&two, "b", Overall::Fail, 2), None);
    }

    #[test]
    fn keep_sets_how_many_stay_running() {
        let all = [
            member("a", 1, None),
            member("b", 2, None),
            member("c", 3, Some(10)),
        ];
        let stop = Some(GroupAction::Stop);
        assert_eq!(step(&all, "b", Overall::Fail, 1), stop);
        assert_eq!(step(&all, "b", Overall::Fail, 2), None);
    }

    #[test]
    fn a_start_time_tie_falls_to_the_name() {
        let all = [member("a", 1, None), member("b", 1, None)];
        let stop = Some(GroupAction::Stop);
        assert_eq!(step(&all, "b", Overall::Fail, 1), stop);
        assert_eq!(step(&all, "a", Overall::Fail, 1), None);
    }

    #[test]
    fn a_job_waits_an_interval_after_any_move() {
        let mut all = [member("a", 1, None), member("b", 2, None)];
        all[0].moved = 97 * SEC;
        assert_eq!(step(&all, "b", Overall::Fail, 1), None);
        all[0].moved = 95 * SEC;
        let stop = Some(GroupAction::Stop);
        assert_eq!(step(&all, "b", Overall::Fail, 1), stop);
    }

    #[test]
    fn the_job_stopped_first_resumes_once_calm_and_settled() {
        let all = [member("a", 1, Some(20)), member("b", 2, Some(30))];
        let go = Some(GroupAction::Continue);
        assert_eq!(step(&all, "a", Overall::Pass, 1), go);
        assert_eq!(step(&all, "b", Overall::Pass, 1), None);
    }

    #[test]
    fn a_resume_waits_for_the_settle_after_the_last_move() {
        let mut all = [member("a", 1, Some(20)), member("b", 2, None)];
        all[1].moved = 80 * SEC;
        assert_eq!(step(&all, "a", Overall::Pass, 1), None);
        all[1].moved = 60 * SEC;
        let go = Some(GroupAction::Continue);
        assert_eq!(step(&all, "a", Overall::Pass, 1), go);
    }

    #[test]
    fn a_resume_waits_for_the_gates_to_have_passed_for_the_settle() {
        let all = [member("a", 1, Some(20)), member("b", 2, None)];
        let turn = Turn {
            members: &all,
            me: "a",
            overall: Overall::Pass,
            calm: false,
            now: 100 * SEC,
            interval: 5 * SEC,
            settle: 30 * SEC,
            keep: 1,
        };
        assert_eq!(crew_step(&turn), None);
    }

    #[test]
    fn other_verdicts_and_strangers_change_nothing() {
        let all = [member("a", 1, None), member("b", 2, Some(10))];
        assert_eq!(step(&all, "a", Overall::Warming, 1), None);
        assert_eq!(step(&all, "b", Overall::Fail, 1), None);
        assert_eq!(step(&all, "a", Overall::Pass, 1), None);
        assert_eq!(step(&all, "zz", Overall::Fail, 1), None);
    }
}
