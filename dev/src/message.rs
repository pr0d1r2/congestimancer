//! The commit-message rule (nix:V8), as pure functions over the message
//! text. RED: signatures only.

/// The lines git keeps: up to the scissors line, `#` comment lines dropped.
#[must_use]
pub(crate) fn kept(_text: &str) -> Vec<&str> {
    Vec::new()
}

/// The first line of the message git keeps.
#[must_use]
pub(crate) fn subject(_text: &str) -> &str {
    ""
}

/// Subject `type(scope): claim` of at most 72 characters, and a body that
/// says why. Merge, revert, fixup and squash subjects are git's and pass.
///
/// # Errors
///
/// The rule broken, with the fix.
pub(crate) fn check(_text: &str) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests;
