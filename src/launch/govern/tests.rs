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
