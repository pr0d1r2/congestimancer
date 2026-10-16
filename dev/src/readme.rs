//! `readme [--check]`: the README badge block, from the files that own
//! each fact (dev:V1).

use crate::badges::{self, Facts};
use crate::facts::{self, Platform};
use crate::io::{Failed, hooks, read, run, update};
use std::path::Path;

/// Why there is nothing to render into.
const NO_BLOCK: &str = "README.md has no <!-- BEGIN badges --> block";

/// Render the badge block into README.md, or with `check` only compare.
///
/// # Errors
///
/// Drift (1), an unreadable or empty source (2).
pub(crate) fn run_verb(repo: &Path, check: bool) -> Result<(), Failed> {
    let facts = gather(repo)?.checked().map_err(|e| (2, e))?;
    let readme = read(repo, "README.md")?;
    let block = badges::render(&facts);
    let wanted = crate::block::splice(&readme, "badges", &block)
        .ok_or((2, NO_BLOCK.to_owned()))?;
    update(repo, "README.md", &wanted, check.then_some("readme"))
}

/// Every fact, each from the file that owns it.
fn gather(repo: &Path) -> Result<Facts, Failed> {
    let cargo = read(repo, "Cargo.toml")?;
    let field = |key: &str| facts::quoted(&cargo, key).unwrap_or_default();
    let (coverage, nixpkgs, platforms) = owned(repo)?;
    Ok(Facts {
        slug: field("repository").replace("https://github.com/", ""),
        license: field("license"),
        edition: field("edition"),
        msrv: field("rust-version"),
        dependencies: facts::dependencies(&cargo),
        coverage,
        steps: hooks(repo)?,
        spec: spec(repo)?,
        nixpkgs,
        platforms,
    })
}

/// What `.coverage`, `flake.lock` and CI's workflow own.
type Owned = (String, (String, String), Vec<Platform>);

/// The facts `.coverage`, `flake.lock` and CI's workflow own.
fn owned(repo: &Path) -> Result<Owned, Failed> {
    let missing = |what: &str| (2, format!("{what} holds no readable value"));
    let coverage = facts::claim(&read(repo, ".coverage")?)
        .ok_or_else(|| missing(".coverage"))?;
    let nixpkgs = facts::nixpkgs(&read(repo, "flake.lock")?)
        .ok_or_else(|| missing("flake.lock nixpkgs"))?;
    let ci = read(repo, ".github/workflows/ci.yml")?;
    let platforms = facts::platforms(&ci).map_err(|e| (2, e))?;
    Ok((coverage, nixpkgs, platforms))
}

/// invariant rows, bug rows and nodes, over every tracked SPEC.md.
fn spec(repo: &Path) -> Result<(usize, usize, usize), Failed> {
    let listed = run(repo, "git", &["ls-files", "-z", "*SPEC.md"])?;
    let texts = listed
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(|p| read(repo, p))
        .collect::<Result<Vec<_>, _>>()?;
    let rows = |prefix, sep| facts::rows(&texts, prefix, sep);
    Ok((rows('V', ':'), rows('B', '|'), texts.len()))
}

#[cfg(test)]
mod tests;
