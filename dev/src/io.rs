//! The only module that touches the repository: git and files, always
//! relative to an explicit repository directory so tests can use scratch
//! repositories.

use std::path::Path;
use std::process::{Command, Output};

/// An exit code and the reason: 1 a broken rule, 2 usage or I/O.
pub(crate) type Failed = (u8, String);

/// Run a program in `repo`; its stdout on success.
///
/// # Errors
///
/// (2, why) when it cannot start or exits non-zero.
pub(crate) fn run(
    repo: &Path,
    program: &str,
    args: &[&str],
) -> Result<String, Failed> {
    let out = spawn(repo, program, args)?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let why =
            format!("{program} {} failed: {}", args.join(" "), err.trim());
        return Err((2, why));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn spawn(repo: &Path, program: &str, args: &[&str]) -> Result<Output, Failed> {
    Command::new(program)
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|e| (2, format!("cannot run {program}: {e}")))
}

/// `git show OBJECT` in `repo`, or empty text when git has no such object
/// (a first commit has no `HEAD~1`).
///
/// # Errors
///
/// (2, why) when git cannot start.
pub(crate) fn show(repo: &Path, object: &str) -> Result<String, Failed> {
    let out = spawn(repo, "git", &["show", object])?;
    if !out.status.success() {
        return Ok(String::new());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The commit message: FILE relative to `repo`, or git's COMMIT_EDITMSG.
///
/// # Errors
///
/// (2, why) when there is no message to judge.
pub(crate) fn message(
    repo: &Path,
    file: Option<&str>,
) -> Result<String, Failed> {
    let path = match file {
        Some(f) => f.to_owned(),
        None => {
            run(repo, "git", &["rev-parse", "--git-path", "COMMIT_EDITMSG"])?
                .trim()
                .to_owned()
        }
    };
    std::fs::read_to_string(repo.join(&path)).map_err(|_| {
        (
            2,
            format!("no message file at {path}, so nothing was checked"),
        )
    })
}

/// A file's text, `path` relative to `repo`.
///
/// # Errors
///
/// (2, why) when it cannot be read.
pub(crate) fn read(repo: &Path, path: &str) -> Result<String, Failed> {
    std::fs::read_to_string(repo.join(path))
        .map_err(|e| (2, format!("cannot read {path}: {e}")))
}

/// Make `path` hold `wanted`: written, or with `check` (the verb that
/// writes it) only compared, drift being (1, why) naming that verb.
/// Equality, never a heuristic.
///
/// # Errors
///
/// Drift under `check`, or a read or write that fails.
pub(crate) fn update(
    repo: &Path,
    path: &str,
    wanted: &str,
    check: Option<&str>,
) -> Result<(), Failed> {
    if read(repo, path)? == wanted {
        return Ok(());
    }
    if let Some(verb) = check {
        let fix = "run `cargo run -p congestimancer-dev --";
        return Err((
            1,
            format!("{path} drifted from its sources; {fix} {verb}`"),
        ));
    }
    std::fs::write(repo.join(path), wanted)
        .map_err(|e| (2, format!("cannot write {path}: {e}")))
}

/// Steps in one hook of `repo`'s hk.pkl, as `pkl` evaluates it.
///
/// # Errors
///
/// (2, why) when `pkl` fails or prints something other than a number.
pub(crate) fn steps(repo: &Path, hook: &str) -> Result<usize, Failed> {
    let expr = format!("hooks[\"{hook}\"].steps.length");
    let text = run(repo, "pkl", &["eval", "-x", &expr, "hk.pkl"])?;
    let text = text.trim();
    text.parse()
        .map_err(|_| (2, format!("pkl printed `{text}` for {expr}")))
}

/// Steps on commit and on push.
///
/// # Errors
///
/// As [`steps`].
pub(crate) fn hooks(repo: &Path) -> Result<(usize, usize), Failed> {
    Ok((steps(repo, "pre-commit")?, steps(repo, "pre-push")?))
}

#[cfg(test)]
mod tests;
