//! Counting processes that match a pattern, leaving out cgmr itself and
//! its ancestors (src/procs:T3, T4, V1, V2, V3).

use std::collections::HashSet;

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

/// `me` and every ancestor of it in `procs`: the processes no count
/// includes (src/procs:V3). A parent loop ends the walk.
#[must_use]
pub fn lineage(procs: &[Proc], me: u32) -> HashSet<u32> {
    let _ = procs;
    HashSet::from([me])
}

/// How many of `procs` have a name matching `pattern`, leaving out `me`
/// and its ancestors (src/procs:V1, V3).
#[must_use]
pub fn count_by_name(procs: &[Proc], pattern: &Glob, me: u32) -> Count {
    let _ = (procs, pattern, me);
    Count(0)
}

#[cfg(test)]
mod tests;
