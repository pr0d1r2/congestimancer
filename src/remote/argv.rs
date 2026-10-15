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

/// The options every remote call takes: never prompt for a password or a
/// host key, fail instead (src/remote:I).
const OPTIONS: [&str; 2] = ["-o", "BatchMode=yes"];

/// The remote command before the gate words; fixed and plain, so it needs
/// no quoting.
const REMOTE: [&str; 3] = ["cgmr", "check", "--json"];

/// `word` quoted for the remote shell: one word, nothing expanded.
///
/// ssh joins its command words with spaces and hands the line to the
/// remote user's shell, so each word is wrapped in `'...'` and each `'`
/// in it becomes `'\''` (src/remote:V5).
#[must_use]
pub fn shell_quote(word: &str) -> String {
    let mut out = String::with_capacity(word.len().saturating_add(2));
    out.push('\'');
    out.push_str(&word.replace('\'', "'\\''"));
    out.push('\'');
    out
}

/// The arguments for [`SSH`] that run `cgmr check --json` with
/// `gate_args` on `host`: every gate word its own argv word, quoted for
/// the remote shell (src/remote:V5).
#[must_use]
pub fn ssh_argv(host: &Host, gate_args: &[String]) -> Vec<String> {
    let fixed = OPTIONS.into_iter().map(str::to_owned);
    let target = std::iter::once(host.as_str().to_owned());
    let remote = REMOTE.into_iter().map(str::to_owned);
    let gates = gate_args.iter().map(|word| shell_quote(word));
    fixed.chain(target).chain(remote).chain(gates).collect()
}

#[cfg(test)]
mod tests;
