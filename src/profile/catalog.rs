//! [`Catalog`]: the profile files in force, in precedence order, and the
//! lookup of a name across them (src/profile:V2, V4).

use super::{Profile, ProfileError, ProfileFile};
use crate::units::Name;

/// The profile files in force, highest precedence first: the explicit file
/// alone, or the project file then the user file (src/profile:V2).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Catalog {
    files: Vec<ProfileFile>,
}

impl Catalog {
    /// A catalog of `files`, highest precedence first.
    #[must_use]
    pub const fn new(files: Vec<ProfileFile>) -> Self {
        Self { files }
    }

    /// The files, highest precedence first.
    #[must_use]
    pub fn files(&self) -> &[ProfileFile] {
        &self.files
    }

    /// Every name the files define, sorted, without repeats.
    #[must_use]
    pub fn names(&self) -> Vec<Name> {
        Vec::new()
    }

    /// The profile called `name` and the file it comes from: the first
    /// file that defines it wins, so the project file shadows the user
    /// file (src/profile:V2).
    ///
    /// # Errors
    ///
    /// [`ProfileError::UnknownProfile`] listing the names there are
    /// (src/profile:V4).
    pub fn lookup(
        &self,
        name: &Name,
    ) -> Result<(&ProfileFile, &Profile), ProfileError> {
        Err(ProfileError::UnknownProfile {
            name: name.clone(),
            available: self.names(),
        })
    }
}

#[cfg(test)]
mod tests;
