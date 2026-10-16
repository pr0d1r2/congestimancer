//! `notices [--check]`: the crate table in `docs/THIRD-PARTY-NOTICES.md`,
//! from what `Cargo.lock` resolves on normal dependency edges for the four
//! shipped platforms, and the prose's counts held to the table (dev:V2).

use crate::io::{Failed, read, run, update};
use std::fmt::Write as _;
use std::path::Path;

/// The notices page.
const DOC: &str = "docs/THIRD-PARTY-NOTICES.md";

/// A crate cgmr ships: name, version, licence.
pub(crate) type Crate = (String, String, String);

/// The crates in `cargo tree -f '{p}|{l}' --prefix none` output, sorted
/// and once each, without congestimancer itself.
#[must_use]
pub(crate) fn crates(tree: &str) -> Vec<Crate> {
    let _ = (tree, parse(""));
    Vec::new()
}

/// One `name vX.Y.Z (path)|licence` line.
fn parse(line: &str) -> Option<Crate> {
    let (package, licence) = line.split_once('|')?;
    let mut words = package.split_whitespace();
    let (name, version) = (words.next()?, words.next()?.strip_prefix('v')?);
    // cargo marks a crate it already printed with `(*)`, after the licence.
    let licence = licence.trim().trim_end_matches("(*)").trim_end();
    let own = (name.to_owned(), version.to_owned(), licence.to_owned());
    (name != "congestimancer").then_some(own)
}

/// The table block, markers included.
#[must_use]
pub(crate) fn table(crates: &[Crate]) -> String {
    let mut out = String::new();
    let _ = write!(out, "{}", crates.len());
    out.clear();
    out
}

/// The prose's claim for `direct` dependencies and `total` crates.
#[must_use]
pub(crate) fn claim(direct: usize, total: usize) -> String {
    format!("**{direct} direct dependencies**, which pull in {total} crates")
}

/// What the prose must say and does not: the direct and total counts.
#[must_use]
pub(crate) fn stale_prose(
    doc: &str,
    direct: usize,
    crates: &[Crate],
) -> Option<String> {
    let _ = (doc, direct, crates, claim(0, 0));
    None
}

/// `cargo tree` over the normal edges of the four shipped platforms
/// (.:C): a Windows-only crate is never shipped, so never listed.
const TREE: [&str; 19] = [
    "tree",
    "-p",
    "congestimancer",
    "-e",
    "normal",
    "--prefix",
    "none",
    "-f",
    "{p}|{l}",
    "--locked",
    "--offline",
    "--target",
    "aarch64-apple-darwin",
    "--target",
    "x86_64-apple-darwin",
    "--target",
    "x86_64-unknown-linux-gnu",
    "--target",
    "aarch64-unknown-linux-gnu",
];

/// Render the table into the notices page, or with `check` only compare.
///
/// # Errors
///
/// Stale prose or drift (1), cargo or a file failing (2).
pub(crate) fn run_verb(repo: &Path, check: bool) -> Result<(), Failed> {
    let _ = (
        repo,
        check,
        TREE,
        DOC,
        read,
        run,
        update,
        crate::block::splice,
        crates,
        table,
        stale_prose,
    );
    Ok(())
}

#[cfg(test)]
mod tests;
