//! The governed jobs of one slot key: a file per job, held by its `flock`
//! while the job lives, so `--govern` jobs can take turns stopping and
//! resuming (src/slot:V13).

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::file::{lock_wait, publish, reap_if_dead, rewrite, suffixes};
use super::{LockDir, SlotError};
use crate::units::Name;

/// Tells apart the crew files of this process.
static NEXT: AtomicU64 = AtomicU64::new(0);

/// One governed job, as its file says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// Its file name after the key's prefix; unique, and the tie-break
    /// between jobs that started together.
    pub name: String,
    /// When it joined, in nanoseconds since the epoch.
    pub started: u64,
    /// When it was stopped, or `None` while it runs.
    pub stopped: Option<u64>,
    /// When it last stopped or resumed; 0 before it first has.
    pub moved: u64,
}

impl Member {
    /// A running member that joined at `now`, named for this process.
    fn new(now: u64) -> Self {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        Self {
            name: format!("{}.{n}", std::process::id()),
            started: now,
            stopped: None,
            moved: 0,
        }
    }

    /// The file text of this member.
    fn text(&self) -> String {
        let stopped = self.stopped.unwrap_or(0);
        format!("{} {stopped} {}\n", self.started, self.moved)
    }

    /// The member `name` that `text` says, or `None` when it does not
    /// parse.
    fn parse(name: &str, text: &str) -> Option<Self> {
        let mut nums = text.split_whitespace().map(str::parse::<u64>);
        let started = nums.next()?.ok()?;
        let stopped = nums.next()?.ok()?;
        let moved = nums.next()?.ok()?;
        Some(Self {
            name: name.to_owned(),
            started,
            stopped: (stopped != 0).then_some(stopped),
            moved,
        })
    }
}

/// This job's place in the crew of a key; its file goes when it drops.
#[derive(Debug)]
pub struct Crew {
    /// The lock dir's path.
    dir: PathBuf,
    /// `+gov.<key>+`.
    prefix: String,
    /// The lock that makes a round one at a time.
    gate: PathBuf,
    /// This job.
    me: Member,
    /// Its open file, locked.
    file: File,
}

impl Crew {
    /// Joins the crew of `key` at `now` (nanoseconds since the epoch).
    ///
    /// # Errors
    ///
    /// [`SlotError::Io`] when the crew file cannot be written.
    pub fn join(
        dir: &LockDir,
        key: &Name,
        now: u64,
    ) -> Result<Self, SlotError> {
        let prefix = format!("+gov.{key}+");
        let me = Member::new(now);
        let file =
            publish(dir.path(), &format!("{prefix}{}", me.name), &me.text())
                .map_err(|e| SlotError::io(dir.path(), &e))?;
        let gate = dir.path().join(format!("+govlock.{key}"));
        let dir = dir.path().to_owned();
        Ok(Self {
            dir,
            prefix,
            gate,
            me,
            file,
        })
    }

    /// This job.
    #[must_use]
    pub const fn me(&self) -> &Member {
        &self.me
    }

    /// Runs `round` alone: no other job of the key reads or writes the
    /// crew meanwhile.
    ///
    /// # Errors
    ///
    /// [`SlotError::Io`] when the key's lock cannot be taken.
    pub fn alone<T>(
        &mut self,
        round: impl FnOnce(&mut Self) -> T,
    ) -> Result<T, SlotError> {
        let io = |e: &io::Error| SlotError::io(&self.gate, e);
        let gate =
            crate::slot::file::open_lock(&self.gate).map_err(|e| io(&e))?;
        lock_wait(&gate).map_err(|e| io(&e))?;
        // The lock goes with `gate` when it drops, after the round.
        Ok(round(self))
    }

    /// Every live job of the key, this one included.
    ///
    /// # Errors
    ///
    /// [`SlotError::Io`] when the lock dir cannot be read.
    pub fn members(&self) -> Result<Vec<Member>, SlotError> {
        let io = |e: &io::Error| SlotError::io(&self.dir, e);
        let names = suffixes(&self.dir, &self.prefix).map_err(|e| io(&e))?;
        let mut found = Vec::new();
        for name in names {
            let path = self.dir.join(format!("{}{name}", self.prefix));
            if let Some(member) = read_live(&path, &name).map_err(|e| io(&e))? {
                found.push(member);
            }
        }
        Ok(found)
    }

    /// Writes that this job is stopped since `at`, or running again when
    /// `None`; `now` is when it moved.
    ///
    /// # Errors
    ///
    /// [`SlotError::Io`] when the crew file cannot be written.
    pub fn mark(
        &mut self,
        stopped: Option<u64>,
        now: u64,
    ) -> Result<(), SlotError> {
        self.me.stopped = stopped;
        self.me.moved = now;
        rewrite(&mut self.file, &self.me.text())
            .map_err(|e| SlotError::io(&self.dir, &e))
    }
}

impl Drop for Crew {
    fn drop(&mut self) {
        let _ = fs::remove_file(
            self.dir.join(format!("{}{}", self.prefix, self.me.name)),
        );
    }
}

/// The member at `path`, or `None` when its job is gone (the file is then
/// removed) or the file does not parse.
fn read_live(path: &Path, name: &str) -> io::Result<Option<Member>> {
    if reap_if_dead(path)? {
        return Ok(None);
    }
    match fs::read_to_string(path) {
        Ok(text) => Ok(Member::parse(name, &text)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests;
