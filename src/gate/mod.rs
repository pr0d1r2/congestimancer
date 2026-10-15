//! `src/gate`: gate model, strict compare, AND evaluation, unsupported verdict, custom gates.
//!
//! Composes only (src:C): each part lives in its own submodule and the
//! public surface is re-exported here and from the crate root (src:V3).
//!
//! The gate takes typed thresholds and a [`Source`](crate::signal::Source);
//! it parses no switch, reads no environment and prints nothing. A caller
//! (`src/cli`, `src/wait`, `src/report`) turns switches into [`Gates`]:
//!
//! - `--load X`, `--run-queue X`: [`Gate::below`] with
//!   [`Gate::per_cpu`] of `cpu_count()`; the `--all-` forms skip it.
//! - `--mem X`: [`Gate::above`] per CPU; `--all-mem` whole;
//!   `--needs-mem M`: [`Gate::above`] with [`Gate::unreserved`] of the
//!   slot ledger's reserved total.
//! - `--mem-level L`, `--cool`, `--on-ac`, `--no-low-power`: [`Gate::is`].
//! - `--between W`: [`Gate::within`].
//! - `--max-procs P=N`, `--max-user-procs U=N`: one [`Gates::push_own`]
//!   per value, each with its own `ProcsSignal`.
//! - `--when CMD`, `--when-file P`: [`Gates::push_custom`].
//!
//! [`evaluate`] runs one round for `run`; [`evaluate_once`] and
//! [`sample_once`] serve `check` and `status`. [`Overall::Unsupported`]
//! is exit 69 (src:V5).

mod custom;
mod eval;
mod model;
mod once;

pub use custom::Custom;
pub use eval::{Checked, Evaluation, Gates, Overall, Subject, evaluate};
pub use model::{Adjust, Gate, Op, Threshold, Verdict};
pub use once::{evaluate_once, sample_once};
