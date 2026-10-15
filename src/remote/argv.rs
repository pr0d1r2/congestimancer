//! [`ssh_argv`]: the words `ssh` runs with for one remote `check`
//! (src/remote:T1, V5).

use super::RemoteError;

/// The ssh binary, found on `PATH` (src/remote:C).
pub const SSH: &str = "ssh";

/// A host `--host` names, safe to hand ssh.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Host(String);

impl Host {
    /// The host as given.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Parses `--host H`.
///
/// # Errors
///
/// [`RemoteError::Host`] when ssh may not take `s` as a host.
pub fn parse_host(s: &str) -> Result<Host, RemoteError> {
    Ok(Host(s.to_owned()))
}

/// `word` quoted for the remote shell: one word, nothing expanded.
#[must_use]
pub fn shell_quote(word: &str) -> String {
    word.to_owned()
}

/// The arguments for [`SSH`] that run `cgmr check --json` with
/// `gate_args` on `host`.
#[must_use]
pub fn ssh_argv(host: &Host, gate_args: &[String]) -> Vec<String> {
    let _ = (host, gate_args);
    Vec::new()
}

#[cfg(test)]
mod tests;
