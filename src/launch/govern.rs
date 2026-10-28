//! `--govern`, the decisions: when the command's process group stops and
//! resumes, and what `cgmr` exits with when the command ends
//! (src/launch:V4). The signalling itself waits on `src/sys`.

use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use crate::gate::Overall;
use crate::slot::Member;

/// What `--govern` does to the command's process group after a round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GroupAction {
    /// `SIGSTOP`: a gate fails.
    Stop,
    /// `SIGCONT`: the gates pass again.
    Continue,
}

/// The action after a round that said `overall`, for a group that is
/// `stopped` or not; `None` leaves it as it is (src/launch:V4).
#[must_use]
pub const fn govern_step(
    stopped: bool,
    overall: Overall,
) -> Option<GroupAction> {
    match overall {
        Overall::Fail if !stopped => Some(GroupAction::Stop),
        Overall::Pass if stopped => Some(GroupAction::Continue),
        _ => None,
    }
}

/// What a job's round knows of its crew (`--govern` jobs of one slot
/// key), all times in nanoseconds on one clock (src/launch:V10).
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct Turn<'a> {
    /// Every live job of the key, this one included.
    pub members: &'a [Member],
    /// Which of them is this job (`Member::name`).
    pub me: &'a str,
    /// What the gates say now.
    pub overall: Overall,
    /// Whether they have passed for `--settle` without a break.
    pub calm: bool,
    /// The time now.
    pub now: u64,
    /// `--interval`.
    pub interval: u64,
    /// `--settle`.
    pub settle: u64,
    /// `--govern-keep`: jobs never stopped below this many running.
    pub keep: u64,
}

/// What a job of a crew does this round (src/launch:V10): when the gates
/// fail, the youngest running job stops, one per interval, never below
/// `keep` running; when they pass, the job stopped first resumes once
/// they have passed for the settle time, and the next only a settle time
/// after that.
#[must_use]
pub fn crew_step(turn: &Turn<'_>) -> Option<GroupAction> {
    let mine = turn.members.iter().find(|m| m.name == turn.me)?;
    match (turn.overall, mine.stopped) {
        (Overall::Fail, None) => stops(turn, mine),
        (Overall::Pass, Some(_)) => resumes(turn, mine),
        _ => None,
    }
}

/// Whether `mine`, running, is the one to stop.
fn stops(turn: &Turn<'_>, mine: &Member) -> Option<GroupAction> {
    let running = turn.members.iter().filter(|m| m.stopped.is_none());
    let left = u64::try_from(running.clone().count()).ok()?;
    let youngest = running.max_by_key(|m| (m.started, &m.name))?;
    let due = since_last_move(turn) >= turn.interval;
    (left > turn.keep && youngest.name == mine.name && due)
        .then_some(GroupAction::Stop)
}

/// Whether `mine`, stopped, is the one to resume.
fn resumes(turn: &Turn<'_>, mine: &Member) -> Option<GroupAction> {
    let stopped = turn.members.iter().filter(|m| m.stopped.is_some());
    let first = stopped.min_by_key(|m| (m.stopped, &m.name))?;
    let due = since_last_move(turn) >= turn.settle;
    (first.name == mine.name && turn.calm && due)
        .then_some(GroupAction::Continue)
}

/// How long since any job of the crew last stopped or resumed.
fn since_last_move(turn: &Turn<'_>) -> u64 {
    let last = turn.members.iter().map(|m| m.moved).max().unwrap_or(0);
    turn.now.saturating_sub(last)
}

/// The exit code that passes on `status`: the command's own, or 128 + n
/// for death by signal n (src/launch:V4).
#[must_use]
pub fn status_code(status: ExitStatus) -> u8 {
    let code = status
        .code()
        .or_else(|| status.signal().and_then(|n| n.checked_add(SIGNAL_BASE)));
    code.and_then(|c| u8::try_from(c).ok()).unwrap_or(u8::MAX)
}

/// What a death by signal n adds n to, as the shell does.
const SIGNAL_BASE: i32 = 128;

#[cfg(test)]
mod tests;
