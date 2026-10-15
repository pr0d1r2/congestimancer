//! `src/wait`: poll loop: interval, backoff, timeout, settle, jitter, heartbeat.
//!
//! Composes only (src:C): each part lives in its own submodule and the
//! public surface is re-exported here and from the crate root (src:V3).
//!
//! The caller (`src/launch`, `src/cli`) turns `--interval`, `--backoff`,
//! `--timeout`, `--settle`, `--jitter` and `--heartbeat` into a
//! [`Pacing`], hands [`poll`] the [`Gates`](crate::gate::Gates), a
//! [`Source`](crate::signal::Source), a [`Clock`](crate::signal::Clock)
//! and stderr in a [`Rig`], and an [`Admit`] for the slot steps
//! (`src/slot` `Waiter`), or [`Open`] when there are none. [`poll`]
//! returns an [`Outcome`]; [`Outcome::exit_code`] says 124 for a timeout
//! and 69 for an unsupported gate.
//!
//! Time (src/wait:V7): every duration here is measured on the
//! [`Clock`](crate::signal::Clock), whose real form is
//! [`std::time::Instant`]: `CLOCK_MONOTONIC` on Linux and
//! `CLOCK_UPTIME_RAW` on macOS. Neither counts time the machine is
//! suspended, so a `--timeout` or `--settle` pauses across a suspend
//! and resumes on wake; the wall clock is never read.

mod pacing;
mod poll;

pub use pacing::{DEFAULT_INTERVAL, MIN_INTERVAL, Pacing};
pub use poll::{Admit, Open, Outcome, Rig, poll};
