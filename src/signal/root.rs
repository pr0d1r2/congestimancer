//! [`Root`]: the filesystem root `/proc`, `/sys` and cgroup paths are read
//! under (src/signal:T5, V5).

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

/// The directory that stands for `/` when reading kernel files.
///
/// On the live machine it is `/` itself ([`Root::system`]); tests and the
/// hidden `CGMR_FIXTURE_ROOT` pass a fixture tree such as
/// `tests/fixtures/<node>/` laid out like the real one
/// (`proc/pressure/cpu`, `sys/class/thermal/...`). It is passed as an
/// argument, never set in the environment or a global (src/signal:V5):
/// `src/sys` and the domain nodes take a `&Root` and call [`Root::join`]
/// for every path they open.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Root(PathBuf);

impl Root {
    /// The live machine's root, `/`.
    #[must_use]
    pub fn system() -> Self {
        Self(PathBuf::from("/"))
    }

    /// A fixture tree that stands for `/`.
    #[must_use]
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self(dir.into())
    }

    /// The directory that stands for `/`.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }

    /// Where the absolute `path` (such as `/proc/pressure/cpu`) lives under
    /// this root.
    ///
    /// The join is lexical and stays inside the root: a leading `/` and
    /// `.` parts are dropped, and `..` never climbs above the root, as in a
    /// chroot. So a cgroup path read from `/proc/self/cgroup` cannot point
    /// a fixture run at the real machine.
    #[must_use]
    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        let mut parts: Vec<&OsStr> = Vec::new();
        for part in path.as_ref().components() {
            match part {
                Component::Normal(name) => parts.push(name),
                Component::ParentDir => {
                    parts.pop();
                }
                Component::RootDir
                | Component::CurDir
                | Component::Prefix(_) => {}
            }
        }
        let mut joined = self.0.clone();
        joined.extend(parts);
        joined
    }
}

/// The live machine's root, `/`.
impl Default for Root {
    fn default() -> Self {
        Self::system()
    }
}

#[cfg(test)]
mod tests;
