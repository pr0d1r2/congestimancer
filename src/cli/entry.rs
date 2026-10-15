//! [`cgmr`]: the entry point `main` calls, and the dispatch from a parsed
//! verb to its code (src:V1, src/cli:V4).

use std::io;
use std::process::ExitCode;

use super::args::{DebugVerb, Verb, parse};
use super::env::Env;
use super::exit::{Failure, OK, USAGE};
use super::meta::{capture_into, completions, man, profiles};
use super::run::run;
use super::verbs::{advise_verb, check, status};
use super::world::{Tty, World};
use crate::remote::SystemSsh;

/// Runs `cgmr` on this process's argv, environment and streams, and
/// returns the exit code (src/cli:V4). `CGMR_FIXTURE_ROOT` is read here,
/// once, and passed down as the root (src/cli:V8).
#[must_use]
pub fn cgmr() -> ExitCode {
    let env = Env::new(std::env::vars_os());
    let (mut out, mut err) = (io::stdout().lock(), io::stderr().lock());
    let mut ssh = SystemSsh;
    let mut world = World::new(env, &mut out, &mut err, &mut ssh);
    world.args = std::env::args_os().collect();
    world.cwd = std::env::current_dir().unwrap_or_default();
    world.tty = Tty::detect();
    ExitCode::from(main(world))
}

/// Parses `world`'s argv and runs the verb; returns the exit code.
pub(crate) fn main(mut world: World<'_>) -> u8 {
    let code = match parse(world.args.clone()) {
        Ok(cli) => dispatch(&mut world, cli.verb),
        Err(err) => Ok(clap_exit(&mut world, &err)),
    };
    let code = code.unwrap_or_else(|failure| {
        world.error(&failure.message);
        failure.code
    });
    let _ = world.out.flush();
    code
}

/// Runs `verb`.
fn dispatch(world: &mut World<'_>, verb: Verb) -> Result<u8, Failure> {
    match verb {
        Verb::Run(args) => run(world, &args),
        Verb::Check(args) => check(world, &args),
        Verb::Status(args) => status(world, &args),
        Verb::Advise(args) => advise_verb(world, &args),
        Verb::Profiles(given) => profiles(world, &given),
        Verb::Completions { shell } => completions(world, shell),
        Verb::Man => man(world),
        Verb::Debug(DebugVerb::Capture { dir }) => capture_into(world, &dir),
    }
}

/// Writes clap's text: help and version to stdout with exit 0, a usage
/// error to stderr with exit 64 (src/cli:V4, V12).
fn clap_exit(world: &mut World<'_>, err: &clap::Error) -> u8 {
    let text = err.render().to_string();
    let (sink, code) = if err.use_stderr() {
        (&mut *world.err, USAGE)
    } else {
        (&mut *world.out, OK)
    };
    let _ = sink.write_all(text.as_bytes());
    code
}

/// What one in-process run of cgmr said, for tests.
#[cfg(test)]
#[derive(Debug, Default)]
pub(crate) struct Ran {
    /// The exit code.
    pub(crate) code: u8,
    /// stdout.
    pub(crate) out: String,
    /// stderr.
    pub(crate) err: String,
}

/// Runs cgmr in-process on `args` (program name added) and `vars`,
/// reaching `--host` through `ssh`; never execs, so tests give no
/// command.
#[cfg(test)]
pub(crate) fn ran(
    args: &[&str],
    vars: &[(&str, &str)],
    ssh: &mut dyn crate::remote::Transport,
) -> Ran {
    use std::ffi::OsString;
    let os = |s: &str| OsString::from(s);
    let env = Env::new(vars.iter().map(|(k, v)| (os(k), os(v))));
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let mut world = World::new(env, &mut out, &mut err, ssh);
    world.args = ["cgmr"].iter().chain(args).map(|a| os(a)).collect();
    let code = main(world);
    let text = |b: Vec<u8>| String::from_utf8_lossy(&b).into_owned();
    Ran {
        code,
        out: text(out),
        err: text(err),
    }
}

#[cfg(test)]
mod tests;
