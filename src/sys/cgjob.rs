//! A delegated cgroup v2 for one `--govern-by cgroup` command: created
//! below the cgroup `cgmr` runs in, entered by the command before exec,
//! frozen and thawed through `cgroup.freeze`, killed through
//! `cgroup.kill` (src/sys:T27, src/launch:V12).

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::GroupSignal;

/// Where cgroup v2 is mounted.
const MOUNT: &str = "/sys/fs/cgroup";

/// A cgroup the command lives in, removed on drop when it is empty.
#[derive(Debug)]
pub struct CgroupJob {
    /// Its directory.
    dir: PathBuf,
}

/// The cgroup `cgmr` runs in, when it may create a child there: the
/// directory below the cgroup2 mount that `/proc/self/cgroup` names and
/// `cgroup.procs` in it is writable.
///
/// # Errors
///
/// [`io::ErrorKind::Unsupported`] with the reason when there is none.
pub fn delegated_cgroup() -> io::Result<PathBuf> {
    Err(unsupported(MOUNT))
}

/// The cgroup directory `proc_cgroup` (the text of `/proc/self/cgroup`)
/// names below `mount`.
///
/// # Errors
///
/// [`io::ErrorKind::Unsupported`] when no line names a cgroup v2.
pub fn own_cgroup_dir(mount: &Path, proc_cgroup: &str) -> io::Result<PathBuf> {
    let _ = (mount, proc_cgroup);
    Err(unsupported("cgroup v2"))
}

impl CgroupJob {
    /// Creates `cgmr-<pid>` below `parent`.
    ///
    /// # Errors
    ///
    /// [`io::ErrorKind::Unsupported`] when `parent` is not a cgroup
    /// `cgmr` may create in; the OS error otherwise.
    pub fn create(parent: &Path, pid: u32) -> io::Result<Self> {
        let _ = (parent, pid);
        Err(unsupported("cgroup"))
    }

    /// The cgroup in directory `dir`, as it is.
    ///
    /// # Errors
    ///
    /// [`io::ErrorKind::Unsupported`] when `dir` has no `cgroup.procs`.
    pub fn at(dir: PathBuf) -> io::Result<Self> {
        let _ = dir;
        Err(unsupported("cgroup"))
    }

    /// Its directory.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Makes the child `command` spawns enter this cgroup before exec,
    /// so nothing it forks is ever outside.
    ///
    /// # Errors
    ///
    /// The OS error of opening `cgroup.procs`.
    pub fn enter_in(&self, command: &mut Command) -> io::Result<()> {
        let _ = command;
        Err(unsupported("cgroup"))
    }

    /// Does what `sig` asks of every process in the cgroup: freezes it,
    /// thaws it, kills it, or signals each process.
    ///
    /// # Errors
    ///
    /// The OS error.
    pub fn signal(&self, sig: GroupSignal) -> io::Result<()> {
        let _ = sig;
        Err(unsupported("cgroup"))
    }
}

/// The error for a cgroup this machine cannot give.
fn unsupported(why: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, why.to_owned())
}

#[cfg(test)]
mod tests;
