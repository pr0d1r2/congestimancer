//! [`check_remote`]: one round of `--host` gates, run over ssh
//! (src/remote:T2, V1).

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
    let stdout = String::from_utf8_lossy(&reply.stdout);
    parse_reply(&stdout).map(Round::Report)
}

#[cfg(test)]
mod tests;
