//! congestimancer-dev: repository tooling (commit-message and changelog
//! rules now; README badges, notices closure and the AGENTS step table
//! later). Never published (dev:C). Re-exports only.

// RED: the modules hold signatures only, so nothing calls them yet. The
// GREEN commit wires them and removes this.
#![expect(dead_code, reason = "RED: wired by the GREEN commit")]

mod changelog;
mod cli;
mod hooks;
mod io;
mod message;

pub use cli::run;
