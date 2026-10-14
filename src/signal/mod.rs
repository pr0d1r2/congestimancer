//! `src/signal`: the common model of every congestion signal: what a
//! reading is, how it is sampled, how tests fake it.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3). Test doubles ([`ManualClock`], [`FakeSource`]) exist only in
//! test builds, for this and every later node's unit tests.

mod clock;
mod id;
mod reading;
mod source;

pub use clock::{Clock, SystemClock};
pub use id::SignalId;
pub use reading::{Level, Measure, Reading, Reason};
pub use source::Source;

#[cfg(test)]
pub(crate) use clock::ManualClock;
#[cfg(test)]
pub(crate) use source::FakeSource;
