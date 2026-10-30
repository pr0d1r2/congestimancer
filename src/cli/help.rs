//! `--help` text: examples for every verb, the exit codes, and the note on
//! a command whose own exit code collides with cgmr's (src/cli:V5, V6).

use super::table::SwitchSpec;

/// One line under the name.
pub(crate) const ABOUT: &str =
    "Gate commands on machine congestion: wait, then run";

/// The tail of `cgmr --help`: an example of every verb, the exit codes,
/// and the collision note (src/cli:V5, V6).
pub(crate) const TOP: &str = "\
Examples:
  cgmr --load 2 -- make -j8          run (the default verb): wait, then exec
  cgmr --load 2 && make              no command: wait, then exit 0
  cgmr check --load 1 --disk 5GB     one-shot, for scripts and agents
  cgmr status --json                 every reading, for agents
  cgmr advise --jobs 8 --needs-mem 2GB --json
  cgmr profiles                      list profiles, their files and switches
  cgmr completions zsh               shell completion script
  cgmr man                           the man page

Every switch --foo-bar also reads CGMR_FOO_BAR, except --when.
Precedence: flag, then environment, then profile, then built-in default.

Exit codes:
  0    gates passed and there is no command; check: every gate passed
  1    check: a gate failed
  64   usage error
  69   a gate is unsupported on this machine
  124  --timeout ran out
  130  SIGINT while waiting; 143 SIGTERM while waiting
  N    once the command runs, its own exit code

Exit code collision: once the command runs, cgmr exits with the
command's own code, so a command that itself exits 64, 69 or 124 looks
like a cgmr usage error, unsupported gate or timeout. To tell them
apart, run `cgmr check` first, or let the command report its own
failures.";

/// `run --help` examples.
pub(crate) const RUN: &str = "\
Examples:
  cgmr --load 2 -- make -j8
  cgmr run --mem 2GB --timeout 10m -- cargo test
  cgmr --slots 2 --slot-key build -- nix build
  cgmr --needs-mem auto:2GiB --mem-key test -- cargo test
  cgmr --load 1 --govern -- ./long-job.sh
  cgmr --load 2 && make

A command that itself exits 64, 69 or 124 looks like a cgmr error;
see `cgmr --help`.";

/// `check --help` examples.
pub(crate) const CHECK: &str = "\
Examples:
  cgmr check --load 1 --disk 5GB
  cgmr check --mem-pressure 10 --json
  cgmr check --host builder --load 2

Exit codes: 0 every gate passed, 1 a gate failed, 64 usage, 69
unsupported.";

/// `status --help` examples.
pub(crate) const STATUS: &str = "\
Examples:
  cgmr status
  cgmr status --json
  cgmr status --disk-path /var/tmp --net-iface en0";

/// `advise --help` examples.
pub(crate) const ADVISE: &str = "\
Examples:
  cgmr advise
  cgmr advise --jobs 8 --needs-mem 2GB
  cgmr advise --jobs 8 --needs-mem 2GB --json

--load and --needs-mem size one job here (defaults 1.0 and 1GiB); they
are not gates.";

/// `profiles --help` examples.
pub(crate) const PROFILES: &str = "\
Examples:
  cgmr profiles
  cgmr profiles --profile-file ./ci.cgmr";

/// `completions --help` examples.
pub(crate) const COMPLETIONS: &str = "\
Examples:
  cgmr completions bash > /etc/bash_completion.d/cgmr
  cgmr completions zsh > ~/.zfunc/_cgmr
  cgmr completions fish > ~/.config/fish/completions/cgmr.fish";

/// `man --help` examples.
pub(crate) const MAN: &str = "\
Examples:
  cgmr man > cgmr.1
  cgmr man | man -l -";

/// The `--help` line of `spec`: its help and its environment variable.
pub(crate) fn switch_help(spec: &SwitchSpec) -> String {
    match spec.env() {
        Some(env) => format!("{} [env: {env}]", spec.help()),
        None => spec.help().to_owned(),
    }
}

#[cfg(test)]
mod tests;
