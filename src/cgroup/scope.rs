//! [`CgroupMode`], [`parse_cgroup_mode`] and [`Scope`]: `--cgroup
//! auto|host|PATH`, and whether gates read a cgroup or the host
//! (src/cgroup:T6, V1).

use super::membership::MOUNT;
use super::{Cgroup, CgroupError, parse_cpu_max, parse_memory_max};
use crate::signal::{Reason, Root};
use crate::sys::read_text;

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
    match s {
        "auto" => Ok(CgroupMode::Auto),
        "host" => Ok(CgroupMode::Host),
        _ if s.starts_with('/') => {
            let below = s
                .strip_prefix(MOUNT)
                .filter(|rest| rest.is_empty() || rest.starts_with('/'));
            Ok(CgroupMode::Path(Cgroup::new(below.unwrap_or(s))))
        }
        _ => Err(CgroupError::Mode {
            input: s.to_owned(),
        }),
    }
}

/// Files whose presence says the process runs in a container: Docker,
/// Podman, and systemd-nspawn and friends.
const CONTAINER_MARKS: [&str; 3] = [
    "/.dockerenv",
    "/run/.containerenv",
    "/run/systemd/container",
];

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
        match mode {
            CgroupMode::Auto => Ok(auto(root)),
            CgroupMode::Host => Ok(host("--cgroup host")),
            CgroupMode::Path(cgroup) if is_cgroup(root, cgroup) => {
                Ok(Self::Cgroup(cgroup.clone()))
            }
            CgroupMode::Path(cgroup) => Err(Reason::from(format!(
                "{} is not a cgroup v2 directory",
                cgroup.dir()
            ))),
        }
    }

    /// The cgroup read, if any.
    #[must_use]
    pub const fn cgroup(&self) -> Option<&Cgroup> {
        match self {
            Self::Cgroup(cgroup) => Some(cgroup),
            Self::Host { .. } => None,
        }
    }
}

/// Host values, because of `reason`.
fn host(reason: impl Into<Reason>) -> Scope {
    Scope::Host {
        reason: reason.into(),
    }
}

/// `--cgroup auto`: this process's cgroup iff it limits the process or
/// the process runs in a container (src/cgroup:V1).
fn auto(root: &Root) -> Scope {
    match Cgroup::detect(root) {
        Ok(cgroup) if is_limited(root, &cgroup) || in_container(root) => {
            Scope::Cgroup(cgroup)
        }
        Ok(_) => host("no cgroup limits this process"),
        Err(reason) => host(reason),
    }
}

/// Whether `cgroup` is a cgroup v2 directory under `root`.
fn is_cgroup(root: &Root, cgroup: &Cgroup) -> bool {
    read_text(root, &cgroup.file("cgroup.controllers")).is_ok()
}

/// Whether `cgroup` or an ancestor caps CPU or memory; an unreadable or
/// garbled limit file caps nothing.
fn is_limited(root: &Root, cgroup: &Cgroup) -> bool {
    let read = |c: &Cgroup, name| read_text(root, &c.file(name)).ok();
    cgroup.lineage().iter().any(|c| {
        let cpu = read(c, "cpu.max")
            .and_then(|t| parse_cpu_max(&t).ok())
            .is_some_and(|max| max.is_limited());
        let mem = read(c, "memory.max")
            .and_then(|t| parse_memory_max(&t).ok())
            .is_some_and(|max| max.is_some());
        cpu || mem
    })
}

/// Whether a container runtime marked this root.
fn in_container(root: &Root) -> bool {
    CONTAINER_MARKS
        .iter()
        .any(|mark| read_text(root, mark).is_ok())
}

#[cfg(test)]
mod tests;
