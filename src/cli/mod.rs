//! `src/cli`: clap verbs, flags, env defaults, exit codes, completions, man page.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3).

mod args;
mod entry;
mod env;
mod exit;
mod gates;
mod help;
mod host;
mod layers;
mod meta;
mod profiles;
mod remote;
mod run;
mod slots;
mod switches;
mod table;
mod values;
mod verbs;
mod world;

pub use entry::cgmr;
pub use switches::SWITCHES;
pub use table::{SwitchSpec, env_name};
