//! The `--when` trust rule: only the user's own file may run commands
//! (src/profile:V3).
//!
//! A cloned repository's `.cgmr` is someone else's text, as an `.envrc` is
//! to direnv; `--when` runs a command, so it is refused there, and in an
//! explicit file too, which a repository's environment can name.

use super::{Origin, ProfileError, ProfileFile};

/// The switch that runs a command.
pub(super) const WHEN: &str = "--when";

/// Refuses the first `--when` in a file that is not the user file.
///
/// # Errors
///
/// [`ProfileError::Untrusted`] naming the line (src/profile:V3).
pub(super) fn check_trust(file: &ProfileFile) -> Result<(), ProfileError> {
    if file.origin() == Origin::User {
        return Ok(());
    }
    let switches = file.profiles().iter().flat_map(|p| p.switches());
    match switches.into_iter().find(|s| s.flag() == WHEN) {
        Some(when) => Err(ProfileError::Untrusted {
            at: when.at().clone(),
        }),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests;
