//! The only module that touches the repository: git and files, always
//! relative to an explicit repository directory so tests can use scratch
//! repositories. RED: signatures only.

use std::path::Path;

/// An exit code and the reason: 1 a broken rule, 2 usage or I/O.
pub(crate) type Failed = (u8, String);

/// Run a program in `repo`; its stdout on success.
///
/// # Errors
///
/// (2, why) when it cannot start or exits non-zero.
pub(crate) fn run(
    _repo: &Path,
    _program: &str,
    _args: &[&str],
) -> Result<String, Failed> {
    Ok(String::new())
}

/// `git show OBJECT` in `repo`, or empty text when git has no such object
/// (a first commit has no `HEAD~1`).
///
/// # Errors
///
/// (2, why) when git cannot start.
pub(crate) fn show(_repo: &Path, _object: &str) -> Result<String, Failed> {
    Ok(String::new())
}

/// The commit message: FILE relative to `repo`, or git's COMMIT_EDITMSG.
///
/// # Errors
///
/// (2, why) when there is no message to judge.
pub(crate) fn message(
    _repo: &Path,
    _file: Option<&str>,
) -> Result<String, Failed> {
    Ok(String::new())
}

#[cfg(test)]
mod tests;
