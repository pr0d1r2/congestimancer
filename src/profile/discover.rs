//! [`discover`]: which profile files are in force, read through a
//! [`ReadText`] the caller lends (src/profile:I, V2, V7).
//!
//! Reading is the caller's: `src/profile` stays pure, and the live reader
//! (the OS facade) plugs in at the CLI. A closure
//! `|p: &Path| std::fs::read_to_string(p)` is one.

use std::io;
use std::path::{Path, PathBuf};

use super::{Catalog, Origin, ProfileError, ProfileFile, parse_profiles};

/// Reads a whole text file.
pub trait ReadText {
    /// The text of the file at `path`.
    ///
    /// # Errors
    ///
    /// The OS error; [`io::ErrorKind::NotFound`] means the file is skipped
    /// (src/profile:V7).
    fn read_text(&self, path: &Path) -> io::Result<String>;
}

/// Any `Fn(&Path) -> io::Result<String>` reads text.
impl<F: Fn(&Path) -> io::Result<String>> ReadText for F {
    fn read_text(&self, path: &Path) -> io::Result<String> {
        self(path)
    }
}

/// The name of the project profile file.
pub const PROJECT_FILE: &str = ".cgmr";

/// Where profile files are looked for (src/profile:I).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Places {
    explicit: Option<PathBuf>,
    cwd: PathBuf,
    config_home: Option<PathBuf>,
    home: Option<PathBuf>,
}

impl Places {
    /// Looking for the project file from `cwd` up; no explicit file and no
    /// user file until they are given.
    #[must_use]
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self {
            cwd: cwd.into(),
            ..Self::default()
        }
    }

    /// The file named by `--profile-file`, else `CGMR_PROFILE_FILE`: read
    /// alone when given (src/profile:V2).
    #[must_use]
    pub fn explicit(mut self, path: Option<PathBuf>) -> Self {
        self.explicit = path;
        self
    }

    /// `$XDG_CONFIG_HOME`, used when it is an absolute path.
    #[must_use]
    pub fn config_home(mut self, path: Option<PathBuf>) -> Self {
        self.config_home = path;
        self
    }

    /// `$HOME`, for `~/.config` when `$XDG_CONFIG_HOME` is unusable.
    #[must_use]
    pub fn home(mut self, path: Option<PathBuf>) -> Self {
        self.home = path;
        self
    }

    /// The user file: `$XDG_CONFIG_HOME/cgmr/profiles`, else
    /// `~/.config/cgmr/profiles`; `None` with neither.
    #[must_use]
    pub fn user_file(&self) -> Option<PathBuf> {
        let xdg = self.config_home.as_ref().filter(|p| p.is_absolute());
        let dot_config = || self.home.as_ref().map(|h| h.join(".config"));
        let base = xdg.cloned().or_else(dot_config)?;
        Some(base.join("cgmr").join("profiles"))
    }
}

/// Reads the profile files in force, highest precedence first.
///
/// The explicit file alone if given; else the nearest `.cgmr` from the
/// working directory up to the filesystem root, then the user file. A file
/// not found is skipped (src/profile:V2, V7).
///
/// # Errors
///
/// [`ProfileError::Read`] for a file that exists but cannot be read
/// (src/profile:V7), or the first parse error of a file read.
pub fn discover(
    places: &Places,
    reader: &impl ReadText,
) -> Result<Catalog, ProfileError> {
    if let Some(path) = &places.explicit {
        let explicit = load(path, Origin::Explicit, reader)?;
        return Ok(Catalog::new(explicit.into_iter().collect()));
    }
    let mut files: Vec<ProfileFile> =
        project(&places.cwd, reader)?.into_iter().collect();
    if let Some(path) = places.user_file() {
        files.extend(load(&path, Origin::User, reader)?);
    }
    Ok(Catalog::new(files))
}

/// The nearest `.cgmr` from `cwd` up to the filesystem root, if any.
fn project(
    cwd: &Path,
    reader: &impl ReadText,
) -> Result<Option<ProfileFile>, ProfileError> {
    for dir in cwd.ancestors() {
        let path = dir.join(PROJECT_FILE);
        if let Some(file) = load(&path, Origin::Project, reader)? {
            return Ok(Some(file));
        }
    }
    Ok(None)
}

/// The file at `path` parsed, or `None` if it is not found
/// (src/profile:V7).
fn load(
    path: &Path,
    origin: Origin,
    reader: &impl ReadText,
) -> Result<Option<ProfileFile>, ProfileError> {
    match reader.read_text(path) {
        Ok(text) => parse_profiles(&text, path, origin).map(Some),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ProfileError::Read {
            path: path.to_owned(),
            reason: e.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests;
