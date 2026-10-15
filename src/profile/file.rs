//! [`parse_profiles`]: a profile file's text into its named switch sets
//! (src/profile:I, V4, V5). Pure: text in, [`ProfileFile`] out.
//!
//! ```text
//! # builds wait for a quiet machine
//! [build]
//! --load 2
//! --mem 4GB
//! ```

use std::path::{Path, PathBuf};

use super::switch::parse_switch;
use super::trust::check_trust;
use super::{At, ProfileError, Switch};
use crate::units::{Name, parse_name};

/// Which kind of file a profile came from; it decides precedence and trust
/// (src/profile:V2, V3).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Origin {
    /// Named by `--profile-file` or `CGMR_PROFILE_FILE`; read alone.
    Explicit,
    /// The nearest `.cgmr` from the working directory up.
    Project,
    /// `$XDG_CONFIG_HOME/cgmr/profiles` (else `~/.config/cgmr/profiles`).
    User,
}

impl Origin {
    /// The word `cgmr profiles` prints for it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::Project => "project",
            Self::User => "user",
        }
    }
}

/// One `[name]` section: a named set of switches.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Profile {
    name: Name,
    at: At,
    switches: Vec<Switch>,
}

impl Profile {
    /// Its name.
    #[must_use]
    pub const fn name(&self) -> &Name {
        &self.name
    }

    /// Its `[name]` header line.
    #[must_use]
    pub const fn at(&self) -> &At {
        &self.at
    }

    /// Its switches, in file order.
    #[must_use]
    pub fn switches(&self) -> &[Switch] {
        &self.switches
    }
}

/// A parsed profile file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileFile {
    path: PathBuf,
    origin: Origin,
    profiles: Vec<Profile>,
}

impl ProfileFile {
    /// Where it was read from.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Which kind of file it is.
    #[must_use]
    pub const fn origin(&self) -> Origin {
        self.origin
    }

    /// Its profiles, in file order.
    #[must_use]
    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    /// The profile called `name`, if this file defines it.
    #[must_use]
    pub fn get(&self, name: &Name) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name() == name)
    }
}

/// Parses the text of the profile file at `path`.
///
/// Each line is blank, a `#` comment, a `[name]` header, or one switch
/// exactly as on the command line (`--load 2`, `--load=2`), under a
/// header. A value wrapped in one pair of matching quotes loses them, as
/// a shell would drop them. Switches are only split here; their names and
/// values are checked with [`Switch::check`].
///
/// # Errors
///
/// [`ProfileError::Syntax`], [`ProfileError::BadName`] or
/// [`ProfileError::DuplicateProfile`], naming `file:line`
/// (src/profile:V4).
pub fn parse_profiles(
    text: &str,
    path: &Path,
    origin: Origin,
) -> Result<ProfileFile, ProfileError> {
    let mut file = ProfileFile {
        path: path.to_owned(),
        origin,
        profiles: Vec::new(),
    };
    for (line, n) in text.lines().zip(1_usize..) {
        file.read_line(line.trim(), At::new(path, n))?;
    }
    check_trust(&file)?;
    Ok(file)
}

/// Why a switch line cannot stand where it is.
const ORPHAN: &str = "a switch before any [name] header";

/// Why a `[` line is no header.
const OPEN_HEADER: &str = "a header is [name], closed by ]";

impl ProfileFile {
    /// Takes in one trimmed line.
    fn read_line(&mut self, line: &str, at: At) -> Result<(), ProfileError> {
        if line.is_empty() || line.starts_with('#') {
            return Ok(());
        }
        if let Some(rest) = line.strip_prefix('[') {
            return self.open(header(rest, &at)?, at);
        }
        let switch =
            parse_switch(line, at.clone()).map_err(|r| syntax(&at, r))?;
        let profile = self.profiles.last_mut();
        profile
            .ok_or_else(|| syntax(&at, ORPHAN))?
            .switches
            .push(switch);
        Ok(())
    }

    /// Opens the section `name`, headed at `at`.
    fn open(&mut self, name: Name, at: At) -> Result<(), ProfileError> {
        if let Some(first) = self.get(&name) {
            let first = first.at.line();
            return Err(ProfileError::DuplicateProfile { at, name, first });
        }
        let switches = Vec::new();
        self.profiles.push(Profile { name, at, switches });
        Ok(())
    }
}

/// The name in a header line that is `[` then `rest`.
fn header(rest: &str, at: &At) -> Result<Name, ProfileError> {
    let inner = rest
        .strip_suffix(']')
        .ok_or_else(|| syntax(at, OPEN_HEADER))?;
    parse_name(inner.trim()).map_err(|error| ProfileError::BadName {
        at: at.clone(),
        error,
    })
}

/// A syntax error on the line at `at`.
fn syntax(at: &At, reason: &'static str) -> ProfileError {
    ProfileError::Syntax {
        at: at.clone(),
        reason,
    }
}

#[cfg(test)]
mod tests;
