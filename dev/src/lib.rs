//! congestimancer-dev: repository tooling: the commit-message and
//! changelog rules, and the generated README badges, notices table and
//! AGENTS.md step table. Never published
//! (dev:C). Re-exports only.

mod agents;
mod badges;
mod block;
mod changelog;
mod cli;
mod facts;
mod hooks;
mod io;
mod message;
mod notices;
mod readme;

pub use cli::run;
