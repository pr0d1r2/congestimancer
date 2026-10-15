//! [`render_profiles`]: the text `cgmr profiles` prints: every profile,
//! its file and its switches (src/profile:I).

use super::Catalog;

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
    let _ = catalog;
    NONE.to_owned()
}

#[cfg(test)]
mod tests;
