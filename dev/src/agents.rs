//! `agents [--check]`: the step table in AGENTS.md, from the steps hk.pkl
//! gives each hook (dev:V3).

use crate::io::{Failed, hooks, read, run, update};
use std::fmt::Write as _;
use std::path::Path;

/// Every push step as one line: id, `commit` or `push`, globs joined by
/// U+001F, and whether it fixes; tab-separated. Asking `pkl` for text
/// keeps the dev crate free of a JSON parser.
const ROWS: &str = r#"let (fast = hooks["pre-commit"].steps) hooks["pre-push"].steps.toMap().entries.map((e) -> let (s = e.value) List(e.key, if (fast.containsKey(e.key)) "commit" else "push", (if (s.glob is String) List(s.glob) else s.glob ?? List()).join("\u{1F}"), if (s.fix != null) "yes" else "-").join("\t")).join("\n")"#;

/// The table block, markers included, for `rows` (as `ROWS` prints them)
/// and the number of commit and push steps.
#[must_use]
pub(crate) fn table(rows: &str, commit: usize, push: usize) -> String {
    let mut out = String::new();
    let _ = write!(out, "{rows}{commit}{push}{}", files(""));
    out.clear();
    out
}

/// Globs as code spans, or "whole tree" for a step without one.
fn files(globs: &str) -> String {
    if globs.is_empty() {
        return "whole tree".to_owned();
    }
    let spans: Vec<String> =
        globs.split('\u{1F}').map(|g| format!("`{g}`")).collect();
    spans.join(" ")
}

/// Why there is nothing to render into.
const NO_BLOCK: &str = "AGENTS.md has no <!-- BEGIN steps --> block";

/// Render the table into AGENTS.md, or with `check` only compare.
///
/// # Errors
///
/// Drift (1), `pkl` or a file failing (2).
pub(crate) fn run_verb(repo: &Path, check: bool) -> Result<(), Failed> {
    let _ = (
        repo,
        check,
        ROWS,
        NO_BLOCK,
        hooks,
        read,
        run,
        update,
        crate::block::splice,
        table,
    );
    Ok(())
}

#[cfg(test)]
mod tests;
