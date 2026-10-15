//! [`CgroupError`]: why a cgroup file did not parse into what it holds.

use std::fmt;

/// Why cgroup text did not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CgroupError {
    /// A line is not in the form its file takes.
    Malformed {
        /// The file, such as `cpu.max`.
        file: &'static str,
        /// The line as read.
        line: String,
    },
    /// A key the file always carries is absent.
    Missing {
        /// The file, such as `cpu.stat`.
        file: &'static str,
        /// The absent key, such as `throttled_usec`.
        key: &'static str,
    },
    /// The process sits in cgroup v1 hierarchies, which are unsupported
    /// (src/cgroup:V5).
    V1,
    /// `--cgroup` is not `auto`, `host` or an absolute path.
    Mode {
        /// The value as given.
        input: String,
    },
}

impl CgroupError {
    /// A [`CgroupError::Malformed`] for `line` of `file`.
    pub(crate) fn malformed(file: &'static str, line: &str) -> Self {
        Self::Malformed {
            file,
            line: line.to_owned(),
        }
    }
}

impl fmt::Display for CgroupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { file, line } => {
                write!(f, "malformed {file} line: {line}")
            }
            Self::Missing { file, key } => write!(f, "{file} has no {key}"),
            Self::V1 => f.write_str("cgroup v1 is unsupported"),
            Self::Mode { input } => write!(
                f,
                "--cgroup takes auto, host or an absolute cgroup path, \
                 not `{input}`"
            ),
        }
    }
}

impl std::error::Error for CgroupError {}

#[cfg(test)]
mod tests;
