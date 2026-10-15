//! `src/remote`: `--host` evaluation over ssh.
//!
//! Composes only (src:C): the ssh argv in `argv`, the failure kinds in
//! `error`, the JSON reader in `json`, the reply in `reply`, the round
//! in `run`.
//!
//! The caller (`src/cli`) turns `--host H` into a [`Host`] with
//! [`parse_host`], and each round calls [`check_remote`] with
//! [`SystemSsh`] and the gate switches as words, exactly as the user gave
//! them. The gates run on H; slots, reservations and the command stay
//! local (src/remote:V1).

mod argv;
mod error;
#[cfg_attr(not(test), expect(dead_code, reason = "RED: wired in GREEN"))]
mod json;
mod reply;
mod run;

pub use argv::{Host, SSH, parse_host, shell_quote, ssh_argv};
pub use error::RemoteError;
pub use reply::{
    RemoteGate, RemoteReport, RemoteSignal, RemoteVerdict, parse_reply,
};
pub use run::{Round, SshReply, SystemSsh, Transport, check_remote};
