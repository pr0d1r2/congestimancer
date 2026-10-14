//! `commit-msg [FILE]` and `changelog [FILE]`: the commit-msg hook's two
//! rules (nix:V8). The hook passes the file git gave it; by hand it
//! defaults to COMMIT_EDITMSG. RED: signatures only.

use crate::io::Failed;
use std::path::Path;

/// The commit-message style rule.
///
/// # Errors
///
/// The rule broken (1), or no message to judge (2).
pub(crate) fn commit_msg(
    _repo: &Path,
    _file: Option<&str>,
) -> Result<(), Failed> {
    Ok(())
}

/// The changelog rule.
///
/// # Errors
///
/// The rule broken (1), or git or the message failing (2).
pub(crate) fn changelog(
    _repo: &Path,
    _file: Option<&str>,
) -> Result<(), Failed> {
    Ok(())
}

#[cfg(test)]
mod tests;
