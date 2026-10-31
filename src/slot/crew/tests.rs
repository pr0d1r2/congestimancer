use std::fs;
use std::io;

use super::Crew;
use crate::slot::scratch::{Scratch, eventually};
use crate::units::{Name, parse_name};

fn key(k: &str) -> io::Result<Name> {
    parse_name(k).map_err(io::Error::other)
}

fn join(scratch: &Scratch, k: &str, now: u64) -> io::Result<Crew> {
    Crew::join(&scratch.lock_dir()?, &key(k)?, now).map_err(io::Error::other)
}

fn count(crew: &Crew) -> io::Result<usize> {
    Ok(crew.members().map_err(io::Error::other)?.len())
}

#[test]
fn a_crew_lists_its_live_members_and_loses_one_on_drop() -> io::Result<()> {
    let scratch = Scratch::new("crew-live")?;
    let one = join(&scratch, "job", 10)?;
    let two = join(&scratch, "job", 20)?;
    let mut members = one.members().map_err(io::Error::other)?;
    members.sort_by_key(|m| m.started);
    assert_eq!(members.len(), 2);
    assert_eq!(members.first().map(|m| m.started), Some(10));
    assert_eq!(members.last().map(|m| m.started), Some(20));
    drop(two);
    assert_eq!(eventually(|| count(&one), |n| *n == 1)?, 1);
    Ok(())
}

#[test]
fn another_key_is_another_crew_even_when_it_extends_the_name() -> io::Result<()>
{
    let scratch = Scratch::new("crew-keys")?;
    let short = join(&scratch, "a", 1)?;
    let long = join(&scratch, "a.b", 1)?;
    assert_eq!(count(&short)?, 1);
    assert_eq!(count(&long)?, 1);
    Ok(())
}

#[test]
fn a_mark_is_seen_by_the_others() -> io::Result<()> {
    let scratch = Scratch::new("crew-mark")?;
    let mut one = join(&scratch, "job", 10)?;
    let two = join(&scratch, "job", 20)?;
    one.mark(Some(30), 30).map_err(io::Error::other)?;
    let members = two.members().map_err(io::Error::other)?;
    let seen = members.iter().find(|m| m.name == one.me().name);
    assert_eq!(seen.and_then(|m| m.stopped), Some(30));
    assert_eq!(seen.map(|m| m.moved), Some(30));
    one.mark(None, 40).map_err(io::Error::other)?;
    let members = two.members().map_err(io::Error::other)?;
    let seen = members.iter().find(|m| m.name == one.me().name);
    assert_eq!(seen.map(|m| m.stopped), Some(None));
    Ok(())
}

#[test]
fn a_dead_member_is_removed_not_listed() -> io::Result<()> {
    let scratch = Scratch::new("crew-dead")?;
    let one = join(&scratch, "job", 10)?;
    let dead = scratch.path().join("+gov.job+1.0");
    fs::write(&dead, "5 0 5\n")?;
    assert_eq!(count(&one)?, 1);
    assert!(!dead.exists());
    Ok(())
}

#[test]
fn a_file_that_does_not_parse_is_not_a_member() -> io::Result<()> {
    let scratch = Scratch::new("crew-junk")?;
    let one = join(&scratch, "job", 10)?;
    let live = join(&scratch, "job", 11)?;
    fs::write(
        live.dir.join(format!("{}{}", live.prefix, live.me.name)),
        "x",
    )?;
    assert_eq!(count(&one)?, 1);
    Ok(())
}

#[test]
fn a_round_runs_alone_and_returns_its_value() -> io::Result<()> {
    let scratch = Scratch::new("crew-alone")?;
    let mut one = join(&scratch, "job", 10)?;
    let seen = one.alone(|c| c.me().started).map_err(io::Error::other)?;
    assert_eq!(seen, 10);
    Ok(())
}

#[test]
fn a_crew_file_goes_with_its_job() -> io::Result<()> {
    let scratch = Scratch::new("crew-gone")?;
    let one = join(&scratch, "job", 10)?;
    drop(one);
    let names: Vec<_> = fs::read_dir(scratch.path())?
        .filter_map(Result::ok)
        .map(|e| e.file_name())
        .filter(|n| n.to_string_lossy().starts_with("+gov.job+"))
        .collect();
    assert!(names.is_empty(), "{names:?}");
    Ok(())
}
