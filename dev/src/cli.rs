//! Argument dispatch for `congestimancer-dev`.
//!
//! ```text
//! congestimancer-dev commit-msg [FILE]   the commit message style rule
//! congestimancer-dev changelog [FILE]    a behaviour change adds an entry
//! congestimancer-dev readme [--check]    the README badge block
//! congestimancer-dev notices [--check]   the third-party crate table
//! congestimancer-dev agents [--check]    the AGENTS.md step table
//! ```
//!
//! Exit 0 clean, 1 a broken rule, 2 usage or I/O.

use crate::hooks;
use crate::io::Failed;
use std::path::Path;

const USAGE: &str = "usage: congestimancer-dev <commit-msg|changelog> [FILE]\n       congestimancer-dev <readme|notices|agents> [--check]";

/// Run one verb against the repository at `repo`; the exit code.
#[must_use]
pub fn run(args: &[String], repo: &Path) -> u8 {
    match dispatch(args, repo) {
        Ok(()) => 0,
        Err((code, why)) => {
            eprintln!("congestimancer-dev: {why}");
            code
        }
    }
}

fn dispatch(args: &[String], repo: &Path) -> Result<(), Failed> {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["commit-msg", rest @ ..] if rest.len() < 2 => {
            hooks::commit_msg(repo, rest.first().copied())
        }
        ["changelog", rest @ ..] if rest.len() < 2 => {
            hooks::changelog(repo, rest.first().copied())
        }
        [verb @ ("readme" | "notices" | "agents"), rest @ ..] => {
            generate(verb, rest, repo)
        }
        _ => Err((2, USAGE.to_owned())),
    }
}

/// A generator verb, writing its block or with `--check` comparing it.
fn generate(verb: &str, rest: &[&str], repo: &Path) -> Result<(), Failed> {
    let check = match rest {
        [] => false,
        ["--check"] => true,
        _ => return Err((2, USAGE.to_owned())),
    };
    match verb {
        "readme" => crate::readme::run_verb(repo, check),
        "notices" => crate::notices::run_verb(repo, check),
        _ => crate::agents::run_verb(repo, check),
    }
}

#[cfg(test)]
mod tests;
