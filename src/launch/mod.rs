//! `src/launch`: exec, no-cmd exit, govern, background priority, env cleanup.
//!
//! Composes only (src:C): each part lives in its own submodule and the
//! public surface is re-exported here and from the crate root (src:V3).
//!
//! The caller (`src/cli`) waits with [`Slots`] as the
//! [`Admit`](crate::wait::Admit): [`Slots::new`] over a
//! [`Waiter`](crate::slot::Waiter) when a slot switch was given, else
//! [`Slots::none`]. [`after_wait`] turns the
//! [`Outcome`](crate::wait::Outcome) and the [`Priority`] switches into a
//! [`Start`] or an exit code (124, 69); [`launch`] then execs the command
//! with its environment cleaned and its slot locks handed over, and
//! returns only with 0 (no command) or an exec failure code (126, 127).
//!
//! With `--govern`, [`launch_governed`] takes [`launch`]'s place: the
//! command is spawned in its own process group and governed by a
//! [`Judge`] every `--interval` ([`LocalGates`] here; with `--host` the
//! caller's judge of the remote gates), until it ends with its own code.

mod admit;
mod after;
mod crewed;
mod env;
mod exec;
mod govern;
mod governed;
mod governor;
mod priority;
#[cfg(test)]
mod scratch;
mod start;

pub use admit::Slots;
pub use after::{Priority, after_wait};
pub use crewed::{CrewRules, Crewed, wall_nanos};
pub use env::clean_env;
pub use exec::{exec_command, exec_failure_code};
pub use govern::{GroupAction, Turn, crew_step, govern_step, status_code};
pub use governed::{CrewPlan, Governing, launch_governed};
pub use governor::{GovernRig, Group, Judge, LocalGates, Mates, govern};
pub use priority::{BACKGROUND_NICE, background};
pub use start::{Start, launch};
