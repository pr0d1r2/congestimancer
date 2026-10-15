//! `src/advise`: `advise` verb: how many jobs fit now, what limits them, what to do.
//!
//! Composes only (src:C): the fit arithmetic in `fit`, the advice in
//! `decide`, the verb and its exit codes in `verb`, the output in
//! `render`.
//!
//! The logic is pure over readings: the caller (`src/cli`) builds an
//! [`AdviseRequest`] from the machine's facts (`cpu::cpu_count`,
//! `slot::reserved_total`) and the switches (`--load`, `--needs-mem`,
//! `--jobs`, `--interval`), and the other requested gates as
//! [`Gates`](crate::gate::Gates); [`advise`] reads them from a
//! [`Source`](crate::signal::Source).

mod decide;
mod fit;
mod render;
mod verb;

pub use decide::{Advice, AdviceAction, AdviceLimit, AdviseRequest, decide};
pub use fit::{cpu_fit, mem_fit};
pub use render::{write_advice, write_advice_json};
pub use verb::{AdviseUnsupported, advise, advise_exit_code, advise_readings};
