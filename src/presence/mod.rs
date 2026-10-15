//! `src/presence`: user idle time, clock window.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3). macOS idle comes from `crate::sys`'s IOKit call; Linux idle
//! from logind over a hand-written D-Bus codec on the stream `crate::sys`
//! connects (R1). The wall clock is injected, so tests set the time.

mod clock;
mod signal;

pub use clock::{SystemWallClock, WallClock};
pub use signal::PresenceSignal;
