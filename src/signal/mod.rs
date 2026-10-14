//! `src/signal`: the common model of every congestion signal: what a
//! reading is, how it is sampled, how tests fake it.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3). Test doubles ([`ManualClock`], [`FakeSource`]) exist only in
//! test builds, for this and every later node's unit tests.
//!
//! How a domain node plugs in: implement [`Signal`] (what it sees now: a
//! gauge, a counter, or unsupported with a reason), wrap it in
//! [`Sampled`], and the result is a [`Source`] whose counters warm up and
//! reset by one rule (src/signal:V1, V2).

mod clock;
mod id;
mod reading;
mod refresh;
mod root;
mod sampled;
mod sampler;
mod source;
mod trend;

pub use clock::{Clock, SystemClock};
pub use id::SignalId;
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
