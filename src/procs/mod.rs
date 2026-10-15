//! `src/procs`: process counts by name & by user.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3). The machine's process table comes from `crate::sys`
//! ([`LiveTable`]); tests count over `tests/fixtures/procs/` tables.

mod count;
mod error;
#[cfg(test)]
mod fixture;
mod glob;
mod limit;
mod signal;

pub use count::{Proc, count_by_name, lineage};
pub use error::ProcsError;
pub use glob::Glob;
pub use limit::{ProcLimit, parse_proc_limit};
pub use signal::{LiveTable, ProcMatch, ProcessTable, ProcsSignal};
