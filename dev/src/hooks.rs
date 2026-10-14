//! `commit-msg [FILE]` and `changelog [FILE]`: the commit-msg hook's two
//! rules (nix:V8). The hook passes the file git gave it; by hand it
//! defaults to COMMIT_EDITMSG.

use crate::changelog::{self, Sections};
use crate::io::{Failed, message, run, show};
use std::path::Path;

/// Product paths are staged paths under `src/`: added, changed, renamed or
/// deleted.
const STAGED: [&str; 6] = [
    "diff",
    "--cached",
    "--name-only",
    "--diff-filter=ACMRD",
    "--",
    "src",
];

/// The commit-message style rule.
///
/// # Errors
///
/// The rule broken (1), or no message to judge (2).
pub(crate) fn commit_msg(
    repo: &Path,
    file: Option<&str>,
) -> Result<(), Failed> {
    crate::message::check(&message(repo, file)?).map_err(|e| (1, e))
}

/// The changelog rule.
///
/// # Errors
///
/// The rule broken (1), or git or the message failing (2).
pub(crate) fn changelog(repo: &Path, file: Option<&str>) -> Result<(), Failed> {
    let text = message(repo, file)?;
    let subject = crate::message::subject(&text);
    let staged = run(repo, "git", &STAGED)?;
    let product = changelog::product(&staged);
    changelog::check(subject, &product, &sections(repo)?).map_err(|e| (1, e))
}

/// The `[Unreleased]` section staged, at HEAD and at its parent, and HEAD's
/// subject; each empty where git has none (a first commit).
fn sections(repo: &Path) -> Result<Sections, Failed> {
    let at = |rev: &str| {
        show(repo, &format!("{rev}:CHANGELOG.md"))
            .map(|t| changelog::unreleased(&t))
    };
    let head_subject =
        run(repo, "git", &["log", "-1", "--format=%s"]).unwrap_or_default();
    Ok(Sections {
        staged: at("")?,
        head: at("HEAD")?,
        parent: at("HEAD~1")?,
        head_subject: head_subject.trim_end().to_owned(),
    })
}

#[cfg(test)]
mod tests;
