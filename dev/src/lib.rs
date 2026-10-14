//! congestimancer-dev: repository tooling (commit-message and changelog
//! rules now; README badges, notices closure and the AGENTS step table
//! later). Never published (dev:C). Re-exports only.

mod changelog;
mod cli;
mod hooks;
mod io;
mod message;

pub use cli::run;
