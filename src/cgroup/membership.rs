//! [`Cgroup`] and [`parse_proc_cgroup`]: which cgroup v2 this process
//! sits in, read from `/proc/self/cgroup` (src/cgroup:T1, V1, V5).

use super::CgroupError;
use crate::signal::{Reason, Root};
use crate::sys::read_text;

/// Where cgroup2 is mounted.
pub(super) const MOUNT: &str = "/sys/fs/cgroup";

/// What the kernel appends to the path of a removed cgroup.
const DELETED: &str = " (deleted)";

/// A cgroup v2, named by its path below the cgroup2 mount (`/` for the
/// root of the mount, `/user.slice/job` below it).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cgroup {
    /// The path below the mount, always starting with `/`, never ending
    /// with one unless it is `/`.
    path: String,
}

impl Cgroup {
    /// The cgroup at `path` below the cgroup2 mount; a missing leading
    /// `/` and trailing `/` are mended.
    #[must_use]
    pub fn new(path: &str) -> Self {
        let inner = path.trim_matches('/');
        Self {
            path: format!("/{inner}"),
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
        format!("{MOUNT}{}", self.path.trim_end_matches('/'))
    }

    /// The absolute path of one of its files, such as `cpu.max`.
    #[must_use]
    pub fn file(&self, name: &str) -> String {
        format!("{}/{name}", self.dir())
    }

    /// This cgroup and every ancestor below the mount root, innermost
    /// first; the mount root alone when this is it.
    #[must_use]
    pub fn lineage(&self) -> Vec<Self> {
        let mut chain = vec![self.clone()];
        let mut rest = self.path.as_str();
        while let Some((parent, _)) = rest.rsplit_once('/')
            && !parent.is_empty()
        {
            chain.push(Self::new(parent));
            rest = parent;
        }
        chain
    }

    /// The cgroup v2 this process sits in, read under `root`.
    ///
    /// # Errors
    ///
    /// Why there is none, for the user: no `/proc/self/cgroup` (not
    /// Linux), cgroup v1 (src/cgroup:V5), or no cgroup2 mount.
    pub fn detect(root: &Root) -> Result<Self, Reason> {
        let text = read_text(root, FILE)
            .map_err(|err| Reason::from(err.to_string()))?;
        let path = parse_proc_cgroup(&text)
            .map_err(|err| Reason::from(err.to_string()))?;
        let cgroup = Self::new(&path);
        match read_text(root, &cgroup.file("cgroup.controllers")) {
            Ok(_) => Ok(cgroup),
            Err(_) => {
                Err(Reason::from(format!("cgroup2 is not mounted at {MOUNT}")))
            }
        }
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
    let (mut unified, mut v1) = (None, false);
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        match parse_line(line)? {
            Line::Unified(_) if unified.is_some() => {
                return Err(CgroupError::malformed(FILE, line));
            }
            Line::Unified(path) => unified = Some(path),
            Line::Legacy => v1 = true,
        }
    }
    match (v1, unified) {
        (true, _) => Err(CgroupError::V1),
        (false, Some(path)) => Ok(path.to_owned()),
        (false, None) => Err(CgroupError::malformed(FILE, "")),
    }
}

/// The file that names this process's cgroups.
const FILE: &str = "/proc/self/cgroup";

/// One line of `/proc/self/cgroup`.
enum Line<'a> {
    /// `0::<path>`: the cgroup v2 hierarchy.
    Unified(&'a str),
    /// `<id>:<controllers>:<path>` with `id` > 0: a v1 hierarchy.
    Legacy,
}

/// Parses `id:controllers:path`.
fn parse_line(line: &str) -> Result<Line<'_>, CgroupError> {
    let mut parts = line.splitn(3, ':');
    let (id, controllers, path) = parts
        .next()
        .and_then(|id| id.parse::<u32>().ok())
        .zip(parts.next())
        .zip(parts.next().filter(|p| p.starts_with('/')))
        .map(|((id, c), p)| (id, c, p))
        .ok_or_else(|| CgroupError::malformed(FILE, line))?;
    if id == 0 && controllers.is_empty() {
        Ok(Line::Unified(path.strip_suffix(DELETED).unwrap_or(path)))
    } else {
        Ok(Line::Legacy)
    }
}

#[cfg(test)]
mod tests;
