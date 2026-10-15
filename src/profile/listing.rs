//! [`render_profiles`]: the text `cgmr profiles` prints: every profile,
//! its file and its switches (src/profile:I).

use super::{Catalog, Profile, ProfileFile, Switch};

/// What `cgmr profiles` prints when no profile file exists.
pub(super) const NONE: &str = "no profiles: no profile file was found\n";

/// The `cgmr profiles` listing of `catalog`.
///
/// One block per file, highest precedence first: the file and its origin,
/// then one line per profile, `[name]` and its switches as they would be
/// written on the command line. A name an earlier file also defines is
/// marked shadowed, naming the file that wins (src/profile:V2).
///
/// ```text
/// /repo/.cgmr (project)
///   [build] --load 2
/// /me/.config/cgmr/profiles (user)
///   [build] --load 8 (shadowed by /repo/.cgmr)
///   [night] --when "test -f /tmp/night"
/// ```
#[must_use]
pub fn render_profiles(catalog: &Catalog) -> String {
    let files = catalog.files();
    if files.is_empty() {
        return NONE.to_owned();
    }
    let mut out = String::new();
    for (i, file) in files.iter().enumerate() {
        let (path, origin) = (file.path().display(), file.origin().as_str());
        out.push_str(&format!("{path} ({origin})\n"));
        let earlier = files.get(..i).unwrap_or_default();
        for profile in file.profiles() {
            out.push_str(&line(profile, earlier));
        }
    }
    out
}

/// One profile's line: `[name]`, its switches, and what shadows it.
fn line(profile: &Profile, earlier: &[ProfileFile]) -> String {
    let mut words = vec![format!("[{}]", profile.name())];
    words.extend(profile.switches().iter().map(written));
    let winner = earlier.iter().find(|f| f.get(profile.name()).is_some());
    if let Some(winner) = winner {
        words.push(format!("(shadowed by {})", winner.path().display()));
    }
    format!("  {}\n", words.join(" "))
}

/// A switch as it would be written: a value that is empty or holds a space
/// is quoted.
fn written(switch: &Switch) -> String {
    let flag = switch.flag();
    match switch.value() {
        None => flag.to_owned(),
        Some(v) if v.is_empty() || v.contains(char::is_whitespace) => {
            format!("{flag} \"{v}\"")
        }
        Some(v) => format!("{flag} {v}"),
    }
}

#[cfg(test)]
mod tests;
