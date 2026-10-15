//! [`advise`]: the verb, from a [`Source`] to an [`Advice`] or exit 69
//! (src/advise:T3, V6).

use std::fmt;

use super::{Advice, AdviseRequest, decide};
use crate::gate::{Checked, Gates};
use crate::signal::{Clock, Reading, Reason, Source};

/// A reading or gate `advise` has no source for here: exit 69 (src:V5).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct AdviseUnsupported {
    /// The switch whose signal has no source, without `--`.
    pub switch: String,
    /// Why, for the user.
    pub reason: Reason,
}

impl AdviseUnsupported {
    /// The exit code of an unsupported gate (src:V5).
    pub const EXIT: u8 = 69;
}

/// `--<switch>: unsupported: <reason>`.
impl fmt::Display for AdviseUnsupported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f;
        Ok(())
    }
}

/// Advises from one round of readings: the whole machine's load average
/// (`SignalId::Load`), its available memory (`SignalId::Mem`), and every
/// other requested gate's verdict.
///
/// # Errors
///
/// [`AdviseUnsupported`] when a gate, the load or the memory has no
/// source here.
pub fn advise_readings(
    request: &AdviseRequest,
    load: &Reading,
    available: &Reading,
    gates: &[Checked],
) -> Result<Advice, AdviseUnsupported> {
    let _ = (load, available, gates, decide, request);
    Err(AdviseUnsupported {
        switch: String::new(),
        reason: Reason::from(""),
    })
}

/// The `advise` verb: evaluates `gates` once, reads the load and the
/// available memory from `source`, and advises. Waits only the
/// request's one `interval`, and only when a gate reads a rate
/// (src/advise:V6).
///
/// # Errors
///
/// [`AdviseUnsupported`] as [`advise_readings`].
pub fn advise<S, C>(
    request: &AdviseRequest,
    gates: &mut Gates,
    source: &mut S,
    clock: &mut C,
) -> Result<Advice, AdviseUnsupported>
where
    S: Source + ?Sized,
    C: Clock + ?Sized,
{
    let _ = (gates, source, clock);
    advise_readings(request, &Reading::Warming, &Reading::Warming, &[])
}

/// The exit code of an `advise` run: 0 for any advice, even `wait`;
/// 69 when something has no source here (src/advise:V6).
#[must_use]
pub const fn advise_exit_code(
    result: &Result<Advice, AdviseUnsupported>,
) -> u8 {
    let _ = result;
    1
}

#[cfg(test)]
mod tests;
