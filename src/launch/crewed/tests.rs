use std::io;
use std::time::Duration;

use super::{CrewRules, Crewed, wall_nanos};
use crate::gate::Overall;
use crate::launch::scratch::Scratch;
use crate::launch::{GroupAction, Mates};
use crate::slot::Crew;
use crate::units::parse_name;

const TICK: Duration = Duration::from_millis(1);

fn rules(keep: u64) -> CrewRules {
    CrewRules {
        interval: TICK,
        settle: TICK,
        keep,
    }
}

fn join(scratch: &Scratch, at: u64, keep: u64) -> io::Result<Crewed> {
    let key = parse_name("job").map_err(io::Error::other)?;
    let crew = Crew::join(scratch.dir(), &key, at).map_err(io::Error::other)?;
    Ok(Crewed::new(crew, rules(keep)))
}

/// A round that did what it was asked; the action it was asked for.
fn round(
    job: &mut Crewed,
    overall: Overall,
    calm: Duration,
) -> io::Result<Option<GroupAction>> {
    let mut asked = None;
    job.round(overall, calm, &mut |a| {
        asked = Some(a);
        true
    })?;
    Ok(asked)
}

fn pause() {
    std::thread::sleep(Duration::from_millis(5));
}

#[test]
fn the_youngest_stops_and_the_last_running_job_never_does() -> io::Result<()> {
    let scratch = Scratch::new("crewed-stop")?;
    let now = wall_nanos();
    let mut old = join(&scratch, now.saturating_sub(2_000), 1)?;
    let mut young = join(&scratch, now.saturating_sub(1_000), 1)?;
    let fail = Overall::Fail;
    assert_eq!(round(&mut old, fail, Duration::ZERO)?, None);
    assert_eq!(
        round(&mut young, fail, Duration::ZERO)?,
        Some(GroupAction::Stop)
    );
    pause();
    assert_eq!(round(&mut old, fail, Duration::ZERO)?, None);
    Ok(())
}

#[test]
fn a_stop_the_group_refused_is_not_recorded() -> io::Result<()> {
    let scratch = Scratch::new("crewed-refused")?;
    let now = wall_nanos();
    let mut old = join(&scratch, now.saturating_sub(2_000), 1)?;
    let mut young = join(&scratch, now.saturating_sub(1_000), 1)?;
    young.round(Overall::Fail, Duration::ZERO, &mut |_| false)?;
    pause();
    assert_eq!(round(&mut old, Overall::Fail, Duration::ZERO)?, None);
    assert_eq!(
        round(&mut young, Overall::Fail, Duration::ZERO)?,
        Some(GroupAction::Stop)
    );
    Ok(())
}

/// A job of key `job` that joined `ago` ns back, taking turns by `rules`.
fn joined(scratch: &Scratch, ago: u64, rules: CrewRules) -> io::Result<Crewed> {
    let key = parse_name("job").map_err(io::Error::other)?;
    let at = wall_nanos().saturating_sub(ago);
    let crew = Crew::join(scratch.dir(), &key, at);
    Ok(Crewed::new(crew.map_err(io::Error::other)?, rules))
}

/// Three jobs, started 3000, 2000 and 1000 ns ago, that settle for
/// `settle`; the youngest two stopped, in that order.
fn two_stopped(
    scratch: &Scratch,
    settle: Duration,
) -> io::Result<(Crewed, Crewed, Crewed)> {
    let keep = 1;
    let rules = CrewRules {
        interval: TICK,
        settle,
        keep,
    };
    let a = joined(scratch, 3_000, rules)?;
    let (mut b, mut c) = (
        joined(scratch, 2_000, rules)?,
        joined(scratch, 1_000, rules)?,
    );
    assert_eq!(round(&mut c, Overall::Fail, NONE)?, Some(GroupAction::Stop));
    pause();
    assert_eq!(round(&mut b, Overall::Fail, NONE)?, Some(GroupAction::Stop));
    Ok((a, b, c))
}

const NONE: Duration = Duration::ZERO;
const CALM: Duration = Duration::from_secs(1);

#[test]
fn the_job_stopped_first_resumes_first() -> io::Result<()> {
    let scratch = Scratch::new("crewed-first")?;
    let (mut a, mut b, mut c) = two_stopped(&scratch, TICK)?;
    pause();
    assert_eq!(round(&mut b, Overall::Pass, CALM)?, None);
    assert_eq!(round(&mut a, Overall::Pass, CALM)?, None);
    assert_eq!(
        round(&mut c, Overall::Pass, CALM)?,
        Some(GroupAction::Continue)
    );
    Ok(())
}

#[test]
fn a_resume_restarts_the_settle_clock_for_the_next() -> io::Result<()> {
    let scratch = Scratch::new("crewed-settle")?;
    let (_a, mut b, mut c) = two_stopped(&scratch, Duration::from_millis(200))?;
    // Not yet: the settle time has not passed since b's stop.
    assert_eq!(round(&mut c, Overall::Pass, CALM)?, None);
    std::thread::sleep(Duration::from_millis(250));
    let go = Some(GroupAction::Continue);
    assert_eq!(round(&mut c, Overall::Pass, CALM)?, go);
    assert_eq!(round(&mut b, Overall::Pass, CALM)?, None);
    std::thread::sleep(Duration::from_millis(250));
    assert_eq!(round(&mut b, Overall::Pass, CALM)?, go);
    Ok(())
}

#[test]
fn a_job_that_resumed_by_signal_runs_in_the_crew() -> io::Result<()> {
    let scratch = Scratch::new("crewed-signal")?;
    let mut old = joined(&scratch, 2_000, rules(1))?;
    let mut young = joined(&scratch, 1_000, rules(1))?;
    let fail = Overall::Fail;
    assert_eq!(round(&mut young, fail, NONE)?, Some(GroupAction::Stop));
    young.resumed();
    pause();
    assert_eq!(round(&mut old, fail, NONE)?, None);
    assert_eq!(round(&mut young, fail, NONE)?, Some(GroupAction::Stop));
    Ok(())
}
