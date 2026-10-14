//! The changelog rule (nix:V8): a `feat`, `fix` or `perf` commit that
//! changes product code under `src/` adds an entry under `## [Unreleased]`
//! in CHANGELOG.md. RED: signatures only.

/// Whether a subject names a change consumers see: `feat`, `fix`, `perf`.
#[must_use]
pub(crate) fn behaviour(_subject: &str) -> bool {
    false
}

/// The staged paths under `src/` that are product code; specs and test
/// files are not.
#[must_use]
pub(crate) fn product(_staged: &str) -> Vec<&str> {
    Vec::new()
}

/// The `## [Unreleased]` section of a changelog, heading excluded.
#[must_use]
pub(crate) fn unreleased(_changelog: &str) -> String {
    String::new()
}

/// The `[Unreleased]` sections the rule compares, and HEAD's subject.
#[derive(Debug, Default)]
pub(crate) struct Sections {
    /// As staged.
    pub(crate) staged: String,
    /// At HEAD.
    pub(crate) head: String,
    /// At HEAD's parent.
    pub(crate) parent: String,
    /// HEAD's subject.
    pub(crate) head_subject: String,
}

/// Pass when the change is not a behaviour change, touches no product
/// code, adds an entry, or amends a commit that brought its own entry.
///
/// # Errors
///
/// The rule broken, listing the product paths and the fix.
pub(crate) fn check(
    _subject: &str,
    _product: &[&str],
    _sections: &Sections,
) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests;
