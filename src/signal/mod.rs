//! `src/signal`: the common model of every congestion signal: what a
//! reading is, how it is sampled, how tests fake it.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3). Test doubles (`ManualClock`, `FakeSource`) exist only in
//! test builds, for this and every later node's unit tests.
//!
//! How a domain node (`src/cpu`, `src/io`, ...) plugs in:
//!
//! 1. Hold a [`Root`] and open every kernel file as `root.join("/proc/..")`,
//!    so tests point it at `tests/fixtures/<node>/` (src/signal:V5).
//! 2. Implement [`Signal`]: for each [`SignalId`] it owns, return
//!    [`Sample::Gauge`] (a value that stands alone), [`Sample::Counter`]
//!    (a running total, see [`Counter`]), or [`Sample::Unsupported`] with
//!    a reason for the user (src:V5). Pressure files go through
//!    [`parse_psi`].
//! 3. Wrap it in [`Sampled`]: the result is a [`Source`] whose counters
//!    warm up and reset by one rule (src/signal:V1, V2).
//!
//! The caller turns gate switches into a [`SignalSet`], reads them with
//! [`refresh()`] at one [`Clock`] instant (src/signal:V3, V4), and keeps
//! [`Trends`] for `--verbose` (src/signal:V7). Every [`Reading`] carries
//! its unit in a [`Measure`], so a report renders it alone (src/signal:V6).

mod clock;
mod id;
mod psi;
mod reading;
mod refresh;
mod root;
mod sampled;
mod sampler;
mod source;
mod trend;

pub use clock::{Clock, SystemClock};
pub use id::SignalId;
pub use psi::{Psi, PsiError, PsiLine, parse_psi};
pub use reading::{Level, Measure, Reading, Reason};
pub use refresh::{SignalSet, refresh};
pub use root::Root;
pub use sampled::{Sample, Sampled, Signal};
pub use sampler::{Counter, Sampler};
pub use source::Source;
pub use trend::{Trend, Trends};

#[cfg(test)]
pub(crate) use clock::ManualClock;
#[cfg(test)]
pub(crate) use source::FakeSource;

#[cfg(test)]
mod tests;
