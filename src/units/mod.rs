//! `src/units`: parse & render sizes, rates, percents, durations,
//! temperatures, clock windows, counts, decimals and names.
//!
//! Composes only (src:C): each value lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3).

mod error;
mod number;

pub use error::UnitError;
