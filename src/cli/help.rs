//! `--help` text: examples for every verb, the exit codes, and the note on
//! a command whose own exit code collides with cgmr's (src/cli:V5, V6).

use super::table::SwitchSpec;

/// One line under the name.
pub(crate) const ABOUT: &str =
    "Gate commands on machine congestion: wait, then run";

/// The tail of `cgmr --help`: an example of every verb, the exit codes,
/// and the collision note (src/cli:V5, V6).
pub(crate) const TOP: &str = "";

/// `run --help` examples.
pub(crate) const RUN: &str = "";

/// `check --help` examples.
pub(crate) const CHECK: &str = "";

/// `status --help` examples.
pub(crate) const STATUS: &str = "";

/// `advise --help` examples.
pub(crate) const ADVISE: &str = "";

/// `profiles --help` examples.
pub(crate) const PROFILES: &str = "";

/// `completions --help` examples.
pub(crate) const COMPLETIONS: &str = "";

/// `man --help` examples.
pub(crate) const MAN: &str = "";

/// The `--help` line of `spec`: its help and its environment variable.
pub(crate) fn switch_help(spec: &SwitchSpec) -> String {
    match spec.env() {
        Some(env) => format!("{} [env: {env}]", spec.help()),
        None => spec.help().to_owned(),
    }
}

#[cfg(test)]
mod tests;
