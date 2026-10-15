//! Counting processes that match a pattern, leaving out cgmr itself and
//! its ancestors (src/procs:T3, T4, V1, V2, V3).

use std::collections::{HashMap, HashSet};

use super::Glob;
use crate::units::Count;

/// One process as the counts see it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Proc {
    /// The process name.
    pub name: String,
    /// The owner's uid, when the OS tells.
    pub uid: Option<u32>,
    /// The process id, when known.
    pub pid: Option<u32>,
    /// The parent's process id, when known.
    pub parent: Option<u32>,
}

impl Proc {
    /// A process called `name`, owned by `uid`, with no known lineage.
    #[must_use]
    pub fn new(name: impl Into<String>, uid: Option<u32>) -> Self {
        Self {
            name: name.into(),
            uid,
            pid: None,
            parent: None,
        }
    }

    /// The same process with its id and its parent's id.
    #[must_use]
    pub const fn with_lineage(mut self, pid: u32, parent: u32) -> Self {
        self.pid = Some(pid);
        self.parent = Some(parent);
        self
    }
}

/// `me` and every ancestor of it, as far as `procs` names parents: the
/// processes no count includes (src/procs:V3). A parent loop ends the
/// walk.
#[must_use]
pub fn lineage(procs: &[Proc], me: u32) -> HashSet<u32> {
    let parents: HashMap<u32, u32> = procs
        .iter()
        .filter_map(|p| Some((p.pid?, p.parent?)))
        .collect();
    let mut found = HashSet::from([me]);
    let mut at = me;
    while let Some(&parent) = parents.get(&at) {
        if !found.insert(parent) {
            break;
        }
        at = parent;
    }
    found
}

/// How many of `procs` have a name matching `pattern`, leaving out `me`
/// and its ancestors (src/procs:V1, V3).
#[must_use]
pub fn count_by_name(procs: &[Proc], pattern: &Glob, me: u32) -> Count {
    count_where(procs, me, |p| pattern.matches(&p.name))
}

/// How many of `procs` are owned by a user whose name matches `pattern`,
/// leaving out `me` and its ancestors (src/procs:V2, V3). `user_name`
/// names a uid; a process with no uid or an unnamed one never matches.
#[must_use]
pub fn count_by_user(
    procs: &[Proc],
    pattern: &Glob,
    me: u32,
    mut user_name: impl FnMut(u32) -> Option<String>,
) -> Count {
    let mut matched: HashMap<u32, bool> = HashMap::new();
    count_where(procs, me, |p| {
        p.uid.is_some_and(|uid| {
            *matched.entry(uid).or_insert_with(|| {
                user_name(uid).is_some_and(|name| pattern.matches(&name))
            })
        })
    })
}

/// How many of `procs` outside `me`'s lineage pass `keep`.
fn count_where(
    procs: &[Proc],
    me: u32,
    mut keep: impl FnMut(&Proc) -> bool,
) -> Count {
    let skip = lineage(procs, me);
    let outside = |p: &&Proc| p.pid.is_none_or(|pid| !skip.contains(&pid));
    let n = procs.iter().filter(outside).filter(|p| keep(p)).count();
    Count(u64::try_from(n).unwrap_or(u64::MAX))
}

#[cfg(test)]
mod tests;
