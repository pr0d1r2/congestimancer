//! congestimancer-dev: repository tooling: the commit-message and
//! changelog rules, and the generated README badges. Never published
//! (dev:C). Re-exports only.

mod badges;
mod block;
mod changelog;
mod cli;
mod facts;
mod hooks;
mod io;
mod message;
mod readme;

pub use cli::run;
