//! congestimancer-dev: repository tooling: the commit-message and
//! changelog rules, the generated README badges and notices table. Never published
//! (dev:C). Re-exports only.

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
