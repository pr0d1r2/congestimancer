//! `src/remote`: `--host` evaluation over ssh.
//!
//! Composes only (src:C): the ssh argv in `argv`, the failure kinds in
//! `error`.
//!
//! The caller (`src/cli`) turns `--host H` into a [`Host`] with
//! [`parse_host`] and hands it the gate switches as words; slots,
//! reservations and the command stay local (src/remote:V1).

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod argv;
mod error;

pub use argv::{Host, SSH, parse_host, shell_quote, ssh_argv};
pub use error::RemoteError;

use crate::gate as _;
use crate::report as _;
