//! The commit-message rule (nix:V8), as pure functions over the message
//! text.

/// git's scissors line: `git commit -v` appends the diff below it, and
/// cleanup removes everything from here on.
const SCISSORS: &str = "# ------------------------ >8 ------------------------";

const TYPES: [&str; 9] = [
    "feat", "fix", "docs", "test", "chore", "refactor", "perf", "build", "ci",
];

/// git's own subjects, which pass untouched.
const GIT: [&str; 4] = ["Merge ", "Revert ", "fixup! ", "squash! "];

/// The lines git keeps: up to the scissors line, `#` comment lines dropped.
#[must_use]
pub(crate) fn kept(text: &str) -> Vec<&str> {
    text.lines()
        .take_while(|l| *l != SCISSORS)
        .filter(|l| !l.starts_with('#'))
        .collect()
}

/// The first line of the message git keeps.
#[must_use]
pub(crate) fn subject(text: &str) -> &str {
    kept(text).first().copied().unwrap_or_default()
}

/// Subject `type(scope): claim` of at most 72 characters, and a body that
/// says why. Merge, revert, fixup and squash subjects are git's and pass.
///
/// # Errors
///
/// The rule broken, with the fix.
pub(crate) fn check(text: &str) -> Result<(), String> {
    let lines = kept(text);
    let subject = lines.first().copied().unwrap_or_default();
    if GIT.iter().any(|p| subject.starts_with(p)) {
        return Ok(());
    }
    headline(subject)?;
    if !has_body(&lines) {
        return Err("the body is empty. Say WHY: the diff already shows \
                    what changed."
            .to_owned());
    }
    Ok(())
}

/// Whether any kept line after the subject, trailers aside, has text.
fn has_body(lines: &[&str]) -> bool {
    lines
        .iter()
        .skip(1)
        .filter(|l| !l.starts_with("Co-Authored-By:"))
        .any(|l| !l.trim().is_empty())
}

/// The subject's style and length.
fn headline(subject: &str) -> Result<(), String> {
    if !styled(subject) {
        let types = TYPES.join(" ");
        return Err(format!(
            "subject must be 'type(scope): claim', type one of {types}. \
             Got: {subject}"
        ));
    }
    let n = subject.chars().count();
    if n > 72 {
        return Err(format!(
            "subject is {n} characters, the limit is 72. Say less, or move \
             detail to the body."
        ));
    }
    Ok(())
}

/// `type(scope)!: claim`: a known type, an optional lower-case scope, an
/// optional `!`, then `: ` and a claim that does not start with a space.
fn styled(subject: &str) -> bool {
    let Some(rest) = TYPES.iter().find_map(|t| subject.strip_prefix(t)) else {
        return false;
    };
    let Some(rest) = scoped(rest) else {
        return false;
    };
    let rest = rest.strip_prefix('!').unwrap_or(rest);
    rest.strip_prefix(": ")
        .is_some_and(|claim| !claim.is_empty() && !claim.starts_with(' '))
}

/// The text after an optional `(scope)`; `None` when the scope is malformed.
fn scoped(rest: &str) -> Option<&str> {
    let Some(inner) = rest.strip_prefix('(') else {
        return Some(rest);
    };
    let (scope, after) = inner.split_once(')')?;
    (!scope.is_empty() && scope.chars().all(scope_char)).then_some(after)
}

fn scope_char(c: char) -> bool {
    c.is_ascii_lowercase()
        || c.is_ascii_digit()
        || matches!(c, '/' | '.' | '_' | '-')
}

#[cfg(test)]
mod tests;
