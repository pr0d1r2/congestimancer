//! The facts the README badges quote, each read from the file that owns
//! it (dev:V1). Pure functions over text: the caller reads the repository
//! and hands the text in, so every rule is testable without one.

/// A `key = "value"` line's value, from a TOML file read as text.
#[must_use]
pub(crate) fn quoted(toml: &str, key: &str) -> Option<String> {
    let _ = (toml, key);
    None
}

/// Whether a TOML table header names direct dependencies: `[dependencies]`
/// or a `[target.'cfg(...)'.dependencies]` table.
fn direct_table(line: &str) -> bool {
    line == "[dependencies]"
        || (line.starts_with("[target.") && line.ends_with(".dependencies]"))
}

/// How many direct dependencies a Cargo.toml declares, across
/// `[dependencies]` and every target-specific dependency table.
#[must_use]
pub(crate) fn dependencies(toml: &str) -> usize {
    let _ = (toml, direct_table(""));
    0
}

/// Whether `s` is a TOML bare key with a `=` after it.
fn is_key(s: &str) -> bool {
    let key = s.trim();
    let bare = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '-');
    !key.is_empty() && !key.starts_with('#') && key.chars().all(bare)
}

/// The line coverage `.coverage` claims (nix:V5), as written.
#[must_use]
pub(crate) fn claim(coverage: &str) -> Option<String> {
    let _ = coverage;
    None
}

/// Rows of a spec table across `specs`: lines starting with `prefix`,
/// digits, then `sep`. Every node numbers its own rows from 1, so rows are
/// counted, never distinct ids.
#[must_use]
pub(crate) fn rows(specs: &[String], prefix: char, sep: char) -> usize {
    let _ = (specs, prefix, sep);
    0
}

/// The nixpkgs release series and its locked commit (7 characters), from
/// `flake.lock`'s `"nixpkgs": {` node. An input that follows nixpkgs is
/// `"nixpkgs": [`, which this header never matches.
#[must_use]
pub(crate) fn nixpkgs(lock: &str) -> Option<(String, String)> {
    let _ = lock;
    None
}

/// A platform badge: architecture and operating system.
pub(crate) type Platform = (&'static str, &'static str);

/// Badge order, whatever order the matrix lists runners in.
const ORDER: [Platform; 4] = [
    ("intel", "linux"),
    ("arm", "linux"),
    ("intel", "macos"),
    ("arm", "macos"),
];

/// The platform a GitHub runner label builds on.
fn runner(os: &str) -> Option<Platform> {
    match os {
        o if o.starts_with("ubuntu") && o.ends_with("-arm") => {
            Some(("arm", "linux"))
        }
        o if o.starts_with("ubuntu") => Some(("intel", "linux")),
        "macos-13" => Some(("intel", "macos")),
        o if o.starts_with("macos") => Some(("arm", "macos")),
        _ => None,
    }
}

/// The platforms CI's matrix builds on, in badge order. Not the flake's
/// systems: a system no runner builds is not a claim CI backs.
///
/// # Errors
///
/// No matrix, or a runner this table does not know, named: guessing its
/// platform would badge something nothing checked.
pub(crate) fn platforms(ci: &str) -> Result<Vec<Platform>, String> {
    let _ = (ci, runner(""), ORDER, is_key(""));
    Ok(Vec::new())
}

#[cfg(test)]
mod tests;
