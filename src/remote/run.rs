//! [`check_remote`]: one round of `--host` gates, run over ssh
//! (src/remote:T2, T3, V1, V3, V4).

use std::io;
use std::process::{Command, Stdio};

use super::{Host, RemoteError, RemoteReport, SSH, parse_reply, ssh_argv};
use crate::signal::Reason;

/// What a finished `ssh` run left behind.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct SshReply {
    /// Its exit code; none when a signal killed it.
    pub code: Option<i32>,
    /// What it wrote to stdout: the remote cgmr's JSON.
    pub stdout: Vec<u8>,
    /// What it wrote to stderr: ssh's and the remote cgmr's complaints.
    pub stderr: Vec<u8>,
}

impl SshReply {
    /// A run that exited `code` having written `stdout` and `stderr`.
    #[must_use]
    pub fn new(
        code: Option<i32>,
        stdout: impl Into<Vec<u8>>,
        stderr: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            code,
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
    }
}

/// How a program is run and waited for; tests hand in a fake, so no unit
/// test ever starts `ssh`.
pub trait Transport {
    /// Runs `program` with `argv` (no shell), its stdin empty, and waits.
    ///
    /// # Errors
    ///
    /// When `program` cannot be started.
    fn run(&mut self, program: &str, argv: &[String]) -> io::Result<SshReply>;
}

/// The real [`Transport`]: starts the program from `PATH`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct SystemSsh;

impl Transport for SystemSsh {
    fn run(&mut self, program: &str, argv: &[String]) -> io::Result<SshReply> {
        let out = Command::new(program)
            .args(argv)
            .stdin(Stdio::null())
            .output()?;
        Ok(SshReply::new(out.status.code(), out.stdout, out.stderr))
    }
}

/// What one remote round came to.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Round {
    /// The remote host judged the gates.
    Report(RemoteReport),
    /// ssh could not reach or log in to the host: this round fails and
    /// the wait goes on (src/remote:V3).
    Retry(Reason),
}

/// Runs the gates `gate_args` on `host` once: `ssh -o BatchMode=yes --
/// host cgmr check --json <gates>` (src/remote:I, V1).
///
/// # Errors
///
/// A [`RemoteError`] when the run cannot go on at all, with its exit
/// code.
pub fn check_remote<T: Transport + ?Sized>(
    transport: &mut T,
    host: &Host,
    gate_args: &[String],
) -> Result<Round, RemoteError> {
    let argv = ssh_argv(host, gate_args);
    let reply =
        transport
            .run(SSH, &argv)
            .map_err(|err| RemoteError::Spawn {
                reason: err.to_string(),
            })?;
    judge(host, &reply)
}

/// ssh's own exit code when it fails: unreachable, refused, auth.
const SSH_FAILED: i32 = 255;

/// The shell's exit code for a command it cannot find.
const NOT_FOUND: i32 = 127;

/// cgmr's exit code for a usage error.
const USAGE: i32 = 64;

/// What `reply` means: ssh's failure is one failed round (src/remote:V3);
/// a remote cgmr that is missing (src/remote:V4) or refuses the switches
/// ends the run; anything else is the remote cgmr's JSON.
fn judge(host: &Host, reply: &SshReply) -> Result<Round, RemoteError> {
    let said = last_line(&reply.stderr);
    match reply.code {
        None => Ok(retry(host, said, "killed by a signal")),
        Some(SSH_FAILED) => Ok(retry(host, said, "exit 255")),
        Some(NOT_FOUND) => Err(RemoteError::Missing {
            host: host.as_str().to_owned(),
        }),
        Some(USAGE) => Err(RemoteError::Usage {
            reason: said.unwrap_or_else(|| "usage error".to_owned()),
        }),
        Some(_) => read(reply, said),
    }
}

/// A failed round, in ssh's words when it said any.
fn retry(host: &Host, said: Option<String>, otherwise: &str) -> Round {
    let why = said.unwrap_or_else(|| otherwise.to_owned());
    Round::Retry(Reason::from(format!("ssh {}: {why}", host.as_str())))
}

/// The remote JSON; when unreadable, the error carries the remote
/// stderr's last line, which says why far better than the parser.
fn read(reply: &SshReply, said: Option<String>) -> Result<Round, RemoteError> {
    let stdout = String::from_utf8_lossy(&reply.stdout);
    parse_reply(&stdout)
        .map(Round::Report)
        .map_err(|err| match (err, said) {
            (RemoteError::Reply { reason }, Some(said)) => RemoteError::Reply {
                reason: format!("{reason} ({said})"),
            },
            (err, _) => err,
        })
}

/// The last line of `stderr` with text on it, trimmed.
fn last_line(stderr: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(stderr);
    let line = text.lines().map(str::trim).rfind(|l| !l.is_empty())?;
    Some(line.to_owned())
}

#[cfg(test)]
mod tests;
