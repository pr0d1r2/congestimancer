//! [`CgroupMode`], [`parse_cgroup_mode`] and [`Scope`]: `--cgroup
//! auto|host|PATH`, and whether gates read a cgroup or the host
//! (src/cgroup:T6, V1).

use super::{Cgroup, CgroupError};
use crate::signal::{Reason, Root};

/// What `--cgroup` asks for.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum CgroupMode {
    /// Use this process's cgroup iff it limits the process or the
    /// process runs in a container (the default, src/cgroup:V1).
    #[default]
    Auto,
    /// Always read host values.
    Host,
    /// Read this cgroup.
    Path(Cgroup),
}

/// Parses `--cgroup`: `auto`, `host`, or an absolute cgroup path, below
/// the cgroup2 mount (`/user.slice/job`) or through it
/// (`/sys/fs/cgroup/user.slice/job`).
///
/// # Errors
///
/// [`CgroupError::Mode`] for anything else.
pub fn parse_cgroup_mode(s: &str) -> Result<CgroupMode, CgroupError> {
    Err(CgroupError::Mode {
        input: s.to_owned(),
    })
}

/// Where gates read the signals a cgroup has its own files for.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Scope {
    /// Host-wide values; `reason` says why no cgroup is read.
    Host {
        /// Why, for the user: `--cgroup host`, no cgroup v2, no limits.
        reason: Reason,
    },
    /// The cgroup's own files.
    Cgroup(Cgroup),
}

impl Scope {
    /// What `mode` comes to under `root`.
    ///
    /// # Errors
    ///
    /// A given path that is not a cgroup v2 directory, for the user; on
    /// macOS every path is (src/cgroup:C). `auto` never fails: without a
    /// cgroup v2 it reads the host, and says why.
    pub fn resolve(root: &Root, mode: &CgroupMode) -> Result<Self, Reason> {
        let _ = (root, mode);
        Err(Reason::from("RED"))
    }

    /// The cgroup read, if any.
    #[must_use]
    pub const fn cgroup(&self) -> Option<&Cgroup> {
        None
    }
}

#[cfg(test)]
mod tests;
