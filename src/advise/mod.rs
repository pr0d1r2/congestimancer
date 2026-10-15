//! `src/advise`: `advise` verb: how many jobs fit now, what limits them, what to do.
//!
//! Composes only (src:C): the fit arithmetic in `fit`, the advice in
//! `decide`.

#![expect(unused_imports, reason = "scaffold: build-order edges")]

mod decide;
mod fit;

pub use decide::{Advice, AdviceAction, AdviceLimit, AdviseRequest, decide};
pub use fit::{cpu_fit, mem_fit};

use crate::cpu as _;
use crate::gate as _;
use crate::io as _;
use crate::mem as _;
use crate::report as _;
use crate::signal as _;
use crate::slot as _;
