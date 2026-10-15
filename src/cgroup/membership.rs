//! [`Cgroup`] and [`parse_proc_cgroup`]: which cgroup v2 this process
//! sits in, read from `/proc/self/cgroup` (src/cgroup:T1, V1, V5).

use super::CgroupError;
use crate::signal::{Reason, Root};

/// Where cgroup2 is mounted.
const MOUNT: &str = "/sys/fs/cgroup";

/// A cgroup v2, named by its path below the cgroup2 mount (`/` for the
/// root of the mount, `/user.slice/job` below it).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cgroup {
    /// The path below the mount, always starting with `/`.
    path: String,
}

impl Cgroup {
    /// The cgroup at `path` below the cgroup2 mount.
    #[must_use]
    pub fn new(path: &str) -> Self {
        let _ = path;
        Self {
            path: String::new(),
        }
    }

    /// The path below the cgroup2 mount: `/user.slice/job`.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The cgroup's directory: `/sys/fs/cgroup/user.slice/job`.
    #[must_use]
    pub fn dir(&self) -> String {
        MOUNT.to_owned()
    }

    /// The absolute path of one of its files, such as `cpu.max`.
    #[must_use]
    pub fn file(&self, name: &str) -> String {
        let _ = name;
        String::new()
    }

    /// This cgroup and every ancestor below the mount root, innermost
    /// first; the mount root alone when this is it.
    #[must_use]
    pub fn lineage(&self) -> Vec<Self> {
        Vec::new()
    }

    /// The cgroup v2 this process sits in, read under `root`.
    ///
    /// # Errors
    ///
    /// Why there is none, for the user: no `/proc/self/cgroup` (not
    /// Linux), cgroup v1 (src/cgroup:V5), or no cgroup2 mount.
    pub fn detect(root: &Root) -> Result<Self, Reason> {
        let _ = root;
        Err(Reason::from("RED"))
    }
}

/// The cgroup v2 path in `/proc/self/cgroup` text: the `0::<path>` line.
///
/// # Errors
///
/// [`CgroupError::V1`] when any line names a v1 hierarchy (a hybrid host
/// keeps its controllers there); [`CgroupError::Malformed`] for a line not
/// `id:controllers:/path`, two `0::` lines, or no line at all.
pub fn parse_proc_cgroup(text: &str) -> Result<String, CgroupError> {
    let _ = (text, CgroupError::malformed);
    Err(CgroupError::V1)
}

#[cfg(test)]
mod tests;
