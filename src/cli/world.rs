//! [`World`]: everything a verb reads from or writes to outside cgmr,
//! gathered once by the entry point and passed down, so a verb runs the
//! same against a fixture tree in a test (src/cli:V8).

use std::ffi::OsString;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use super::args::Given;
use super::env::Env;
use super::exit::{FAIL, Failure};
use super::layers::{Resolved, resolve};
use super::profiles::profile_layer;
use crate::remote::Transport;
use crate::report::{Style, Warnings};
use crate::signal::Root;
use crate::sys::read_text;

/// Whether stdout and stderr are terminals.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Tty {
    /// stdout is a terminal.
    pub(crate) out: bool,
    /// stderr is a terminal.
    pub(crate) err: bool,
}

/// What one run of cgmr reads and writes.
pub(crate) struct World<'a> {
    /// argv, program name first.
    pub(crate) args: Vec<OsString>,
    /// The environment.
    pub(crate) env: Env,
    /// The working directory.
    pub(crate) cwd: PathBuf,
    /// Where kernel files are read: `/`, or `CGMR_FIXTURE_ROOT`.
    pub(crate) root: Root,
    /// stdout.
    pub(crate) out: &'a mut dyn Write,
    /// stderr.
    pub(crate) err: &'a mut dyn Write,
    /// Which streams are terminals.
    pub(crate) tty: Tty,
    /// How `--host` reaches another machine.
    pub(crate) ssh: &'a mut dyn Transport,
    /// Whether warnings are hidden (`--quiet`), once known.
    pub(crate) quiet: bool,
}

/// The root kernel files are read under: `CGMR_FIXTURE_ROOT` when set
/// (tests), else `/` (src/cli:V8). Read once, never set.
pub(crate) fn fixture_root(env: &Env) -> Root {
    let fixture = env.var_os(super::env::FIXTURE_ROOT);
    fixture
        .filter(|v| !v.is_empty())
        .map_or_else(Root::system, Root::new)
}

/// Reads a profile file from disk.
pub(crate) fn read_file(path: &Path) -> io::Result<String> {
    std::fs::read_to_string(path)
}

impl Tty {
    /// Whether this process's stdout and stderr are terminals.
    pub(crate) fn detect() -> Self {
        Self {
            out: io::stdout().is_terminal(),
            err: io::stderr().is_terminal(),
        }
    }
}

impl<'a> World<'a> {
    /// A world on `env` writing to `out` and `err`, reaching hosts
    /// through `ssh`; the root comes from `env` (src/cli:V8), argv is
    /// empty, the working directory `.`, and no stream is a terminal.
    pub(crate) fn new(
        env: Env,
        out: &'a mut dyn Write,
        err: &'a mut dyn Write,
        ssh: &'a mut dyn Transport,
    ) -> Self {
        Self {
            args: Vec::new(),
            root: fixture_root(&env),
            env,
            cwd: PathBuf::from("."),
            out,
            err,
            tty: Tty::default(),
            ssh,
            quiet: false,
        }
    }
}

impl World<'_> {
    /// The switches as every layer sets them, checked, with the unknown
    /// variable warnings written (src/cli:V2, V10, V11).
    ///
    /// # Errors
    ///
    /// A usage error: a bad profile or a value that does not parse.
    pub(crate) fn settle(
        &mut self,
        given: &Given,
    ) -> Result<Resolved, Failure> {
        let profile = profile_layer(given, &self.env, &self.cwd, &read_file)?;
        let resolved = resolve(given, &self.env, &profile)?;
        resolved.check()?;
        self.quiet = resolved.on("quiet");
        for warning in self.env.unknown() {
            self.warn(&warning);
        }
        Ok(resolved)
    }

    /// Writes `text` as a warning, unless `--quiet` (src/cli:V12).
    pub(crate) fn warn(&mut self, text: &str) {
        // A warning that cannot be written is not worth failing over.
        let _ = Warnings::new(&mut *self.err, self.quiet).warn(text);
    }

    /// Writes `text` as an error line; `--quiet` never hides it.
    pub(crate) fn error(&mut self, text: &str) {
        let _ = writeln!(self.err, "cgmr: {text}");
    }

    /// The colour style for a stream that is a terminal or not.
    pub(crate) fn style(&self, tty: bool) -> Style {
        Style::choose(tty, self.env.var_os("NO_COLOR"))
    }

    /// This machine's name: the kernel's under the root, else
    /// `$HOSTNAME`, else `localhost`.
    pub(crate) fn hostname(&self) -> String {
        let kernel = read_text(&self.root, "/proc/sys/kernel/hostname").ok();
        let kernel = kernel.map(|t| t.trim().to_owned());
        let env =
            || self.env.text("HOSTNAME").ok().flatten().map(str::to_owned);
        let name = kernel.filter(|n| !n.is_empty()).or_else(env);
        name.unwrap_or_else(|| "localhost".to_owned())
    }
}

/// Wraps a failed write to stdout as a failure, exit 1.
/// A closed pipe (`cgmr man | head`) is no failure.
pub(crate) fn wrote(written: io::Result<()>) -> Result<(), Failure> {
    match written {
        Err(e) if e.kind() != io::ErrorKind::BrokenPipe => {
            Err(Failure::new(FAIL, format!("cannot write: {e}")))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests;
