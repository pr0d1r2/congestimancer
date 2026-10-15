//! [`ProfileError`]: what went wrong in a profile file, named by `file:line`
//! (src/profile:V4, V7).

use std::fmt;
use std::path::{Path, PathBuf};

use crate::units::{Name, UnitError};

/// Where in a profile file a line sits: printed as `file:line`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct At {
    path: PathBuf,
    line: usize,
}

impl At {
    /// Line `line` (counted from 1) of the file at `path`.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, line: usize) -> Self {
        Self {
            path: path.into(),
            line,
        }
    }

    /// The file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The line, counted from 1.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }
}

/// Renders as `file:line`.
impl fmt::Display for At {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.path.display(), self.line)
    }
}

/// A profile could not be read, parsed, trusted or found.
///
/// Every variant is a usage error: the CLI prints it on one line and exits
/// [`ProfileError::EXIT`] (src/profile:V3, V4, V6, V7). Variants about a
/// line name it by `file:line`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    /// The file exists but could not be read (src/profile:V7).
    Read {
        /// The file.
        path: PathBuf,
        /// Why, as the OS said it.
        reason: String,
    },
    /// The line is neither blank, a `#` comment, a `[name]` header nor a
    /// switch under one.
    Syntax {
        /// The line.
        at: At,
        /// What is wrong, in words.
        reason: &'static str,
    },
    /// A `[name]` header whose name is not a safe identifier
    /// (src/units:V13).
    BadName {
        /// The header line.
        at: At,
        /// Why the name is refused.
        error: UnitError,
    },
    /// A second `[name]` header for a name already in the same file.
    DuplicateProfile {
        /// The second header.
        at: At,
        /// The name.
        name: Name,
        /// The line of the first header.
        first: usize,
    },
    /// A switch the CLI does not have (src/profile:V4).
    UnknownSwitch {
        /// The line.
        at: At,
        /// The switch as written.
        flag: String,
    },
    /// A switch whose value the CLI refuses (src/profile:V4, V5).
    BadValue {
        /// The line.
        at: At,
        /// The switch as written.
        flag: String,
        /// Why the value is refused, as the CLI words it.
        reason: String,
    },
    /// A single-value switch set twice in one profile (src/profile:V6).
    RepeatedSwitch {
        /// The second line.
        at: At,
        /// The switch as written.
        flag: String,
        /// The line that set it first.
        first: usize,
    },
    /// `--when` outside the user's own profile file (src/profile:V3).
    Untrusted {
        /// The line.
        at: At,
    },
    /// No profile file defines the name asked for (src/profile:V4).
    UnknownProfile {
        /// The name asked for.
        name: Name,
        /// Every name the files define, sorted, without repeats.
        available: Vec<Name>,
    },
}

impl ProfileError {
    /// The exit code every profile error ends the run with: 64, usage.
    pub const EXIT: u8 = 64;

    /// The line the error is about, if it is about one.
    #[must_use]
    pub const fn at(&self) -> Option<&At> {
        match self {
            Self::Syntax { at, .. }
            | Self::BadName { at, .. }
            | Self::DuplicateProfile { at, .. }
            | Self::UnknownSwitch { at, .. }
            | Self::BadValue { at, .. }
            | Self::RepeatedSwitch { at, .. }
            | Self::Untrusted { at } => Some(at),
            Self::Read { .. } | Self::UnknownProfile { .. } => None,
        }
    }

    /// What went wrong, without the place.
    fn what(&self) -> String {
        match self {
            Self::Read { path, reason } => read(path, reason),
            Self::Syntax { reason, .. } => (*reason).to_owned(),
            Self::BadName { error, .. } => format!("bad profile name: {error}"),
            Self::DuplicateProfile { name, first, .. } => dup(name, *first),
            Self::UnknownSwitch { flag: f, .. } => {
                format!("unknown switch {f}")
            }
            Self::BadValue { flag, reason, .. } => format!("{flag}: {reason}"),
            Self::RepeatedSwitch { flag, first, .. } => repeated(flag, *first),
            Self::Untrusted { .. } => UNTRUSTED.to_owned(),
            Self::UnknownProfile { name, available: a } => unknown(name, a),
        }
    }
}

/// The message for a file that exists but cannot be read.
fn read(path: &Path, reason: &str) -> String {
    format!("cannot read {}: {reason}", path.display())
}

/// The message for a second header of one name.
fn dup(name: &Name, first: usize) -> String {
    format!("profile [{name}] already defined on line {first}")
}

/// The message for a single-value switch set twice.
fn repeated(flag: &str, first: usize) -> String {
    format!("{flag} already set on line {first}")
}

/// Why `--when` is refused outside the user file (src/profile:V3).
const UNTRUSTED: &str = "--when is allowed only in the user profile file: a \
                         project file must not run commands";

/// The message for a name no file defines, listing the ones that exist.
fn unknown(name: &Name, available: &[Name]) -> String {
    if available.is_empty() {
        return format!("unknown profile {name}: no profiles are defined");
    }
    let names: Vec<&str> = available.iter().map(Name::as_str).collect();
    format!("unknown profile {name}; available: {}", names.join(", "))
}

/// Renders as `file:line: what`, or `what` alone when no line is at fault.
impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.at() {
            Some(at) => write!(f, "{at}: {}", self.what()),
            None => f.write_str(&self.what()),
        }
    }
}

impl std::error::Error for ProfileError {}

#[cfg(test)]
mod tests;
