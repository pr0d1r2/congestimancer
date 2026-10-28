//! [`Crewed`]: [`Mates`] over the crew files of a slot key, so `--govern`
//! jobs sharing a key stop one at a time and resume one at a time
//! (src/launch:V10).

use std::io;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{GroupAction, Mates, Turn, crew_step};
use crate::gate::Overall;
use crate::slot::{Crew, Member, SlotError};

/// How a crew takes turns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrewRules {
    /// `--interval`: one stop per interval.
    pub interval: Duration,
    /// `--settle`: one resume per settle time.
    pub settle: Duration,
    /// `--govern-keep`: running jobs never stopped below this many.
    pub keep: u64,
}

/// A job's place in its crew, with the rules it takes turns by.
#[derive(Debug)]
pub struct Crewed {
    /// The crew files.
    crew: Crew,
    /// The rules.
    rules: CrewRules,
}

impl Crewed {
    /// `crew` taking turns by `rules`.
    #[must_use]
    pub const fn new(crew: Crew, rules: CrewRules) -> Self {
        Self { crew, rules }
    }
}

/// Nanoseconds since the epoch, the one clock all jobs of a key share.
#[must_use]
pub fn wall_nanos() -> u64 {
    let since = SystemTime::now().duration_since(UNIX_EPOCH);
    since.map_or(0, |d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
}

/// `d` in nanoseconds, saturating.
fn nanos(d: Duration) -> u64 {
    u64::try_from(d.as_nanos()).unwrap_or(u64::MAX)
}

impl Mates for Crewed {
    fn round(
        &mut self,
        overall: Overall,
        calm: Duration,
        act: &mut dyn FnMut(GroupAction) -> bool,
    ) -> io::Result<()> {
        let rules = self.rules;
        let now = wall_nanos();
        let calm = calm >= rules.settle;
        self.crew
            .alone(|crew| turn(crew, &rules, (overall, calm, now), act))
            .and_then(|done| done)
            .map_err(io::Error::other)
    }

    fn resumed(&mut self) {
        let _ = self.crew.mark(None, wall_nanos());
    }
}

impl CrewRules {
    /// The turn of job `me` among `members` at `(overall, calm, now)`.
    fn turn<'a>(
        &self,
        (members, me): (&'a [Member], &'a str),
        (overall, calm, now): (Overall, bool, u64),
    ) -> Turn<'a> {
        Turn {
            members,
            me,
            overall,
            calm,
            now,
            interval: nanos(self.interval),
            settle: nanos(self.settle),
            keep: self.keep,
        }
    }
}

/// One round of `crew` at `(overall, calm, now)`: what its turn says,
/// done through `act` and written down when it was done.
fn turn(
    crew: &mut Crew,
    rules: &CrewRules,
    at: (Overall, bool, u64),
    act: &mut dyn FnMut(GroupAction) -> bool,
) -> Result<(), SlotError> {
    let members = crew.members()?;
    let me = crew.me().name.clone();
    match crew_step(&rules.turn((&members, &me), at)) {
        Some(action) if act(action) => {
            let stopped = (action == GroupAction::Stop).then_some(at.2);
            crew.mark(stopped, at.2)
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests;
