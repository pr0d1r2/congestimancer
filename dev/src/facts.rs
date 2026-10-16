//! The facts the README badges quote, each read from the file that owns
//! it (dev:V1). Pure functions over text: the caller reads the repository
//! and hands the text in, so every rule is testable without one.

/// A `key = "value"` line's value, from a TOML file read as text.
#[must_use]
pub(crate) fn quoted(toml: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = \"");
    toml.lines()
        .find_map(|l| l.strip_prefix(&prefix)?.strip_suffix('"'))
        .map(str::to_owned)
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
    let mut inside = false;
    let mut count = 0usize;
    for line in toml.lines().map(str::trim) {
        if line.starts_with('[') && line.ends_with(']') {
            inside = direct_table(line);
        } else if inside && line.split('=').next().is_some_and(is_key) {
            count = count.saturating_add(1);
        }
    }
    count
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
    let value = coverage.lines().find_map(|l| l.strip_prefix("lines "))?;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

/// Rows of a spec table across `specs`: lines starting with `prefix`,
/// digits, then `sep`. Every node numbers its own rows from 1, so rows are
/// counted, never distinct ids.
#[must_use]
pub(crate) fn rows(specs: &[String], prefix: char, sep: char) -> usize {
    let row = |l: &&str| {
        l.strip_prefix(prefix).is_some_and(|rest| {
            let tail = rest.trim_start_matches(|c: char| c.is_ascii_digit());
            tail.len() < rest.len() && tail.starts_with(sep)
        })
    };
    specs.iter().map(|s| s.lines().filter(row).count()).sum()
}

/// The nixpkgs release series and its locked commit (7 characters), from
/// `flake.lock`'s `"nixpkgs": {` node. An input that follows nixpkgs is
/// `"nixpkgs": [`, which this header never matches.
#[must_use]
pub(crate) fn nixpkgs(lock: &str) -> Option<(String, String)> {
    let node: Vec<&str> = lock
        .lines()
        .map(str::trim)
        .skip_while(|l| *l != "\"nixpkgs\": {")
        .collect();
    let field = |name: &str| {
        let key = format!("\"{name}\": \"");
        node.iter()
            .find_map(|l| l.strip_prefix(&key)?.split('"').next())
    };
    let series = field("ref")?.strip_prefix("nixos-")?.to_owned();
    Some((series, field("rev")?.get(..7)?.to_owned()))
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
    let list = ci
        .lines()
        .find_map(|l| l.trim().strip_prefix("os: ["))
        .and_then(|l| l.strip_suffix(']'))
        .ok_or("ci.yml has no `os: [...]` matrix")?;
    let found = list.split(',').map(str::trim).map(|os| {
        runner(os).ok_or(format!("ci.yml runner `{os}` has no platform"))
    });
    let found = found.collect::<Result<Vec<_>, _>>()?;
    Ok(ORDER.into_iter().filter(|p| found.contains(p)).collect())
}

#[cfg(test)]
mod tests;
