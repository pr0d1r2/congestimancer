//! `src/units`: parse & render sizes, rates, percents, durations,
//! temperatures, clock windows, counts, decimals and names.
//!
//! Composes only (src:C): each value lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3).

mod duration;
mod error;
mod number;
mod percent;
mod size;

pub use duration::parse_duration;
pub use error::UnitError;
pub use percent::{Percent, parse_percent};
pub use size::{Bytes, Rate, parse_bytes, parse_rate};
