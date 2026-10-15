//! [`advise`]: the verb, from a [`Source`] to an [`Advice`] or exit 69
//! (src/advise:T3, V6).

use std::fmt;

use super::{Advice, AdviseRequest, decide};
use crate::gate::{Checked, Gates, Verdict, evaluate_once, sample_once};
use crate::report::switch_name;
use crate::signal::{
    Clock, Measure, Reading, Reason, SignalId, SignalSet, Source,
};
use crate::units::{Bytes, Decimal};

/// The switch the load reading answers to.
const LOAD: &str = "load";

/// The switch the memory reading answers to.
const NEEDS_MEM: &str = "needs-mem";

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

    /// `switch` has no source, for `reason`.
    fn new(switch: impl Into<String>, reason: Reason) -> Self {
        Self {
            switch: switch.into(),
            reason,
        }
    }
}

/// `--<switch>: unsupported: <reason>`.
impl fmt::Display for AdviseUnsupported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "--{}: unsupported: {}", self.switch, self.reason)
    }
}

/// Advises from one round of readings: the whole machine's load average
/// (`SignalId::Load`), its available memory (`SignalId::Mem`), and every
/// other requested gate's verdict. A load or memory still warming fits
/// nothing.
///
/// # Errors
///
/// [`AdviseUnsupported`] when a gate, the load or the memory has no
/// source here, or reads a unit it cannot be.
pub fn advise_readings(
    request: &AdviseRequest,
    load: &Reading,
    available: &Reading,
    gates: &[Checked],
) -> Result<Advice, AdviseUnsupported> {
    if let Some(err) = gates.iter().find_map(unsupported_gate) {
        return Err(err);
    }
    let overloaded = Decimal::from_thousandths(u64::MAX);
    let load = value(load, LOAD, overloaded, |m| match m {
        Measure::Number(n) => Some(n),
        _ => None,
    })?;
    let available = value(available, NEEDS_MEM, Bytes(0), |m| match m {
        Measure::Bytes(b) => Some(b),
        _ => None,
    })?;
    Ok(decide(request, load, available, gates))
}

/// The error for a gate with no source, if `checked` is one.
fn unsupported_gate(checked: &Checked) -> Option<AdviseUnsupported> {
    match &checked.verdict {
        Verdict::Unsupported { reason } => Some(AdviseUnsupported::new(
            switch_name(&checked.subject),
            reason.clone(),
        )),
        _ => None,
    }
}

/// The value `reading` holds for `switch`: `warming` while it has none,
/// unsupported when it has no source or the wrong unit.
fn value<T>(
    reading: &Reading,
    switch: &str,
    warming: T,
    pick: impl FnOnce(Measure) -> Option<T>,
) -> Result<T, AdviseUnsupported> {
    match reading {
        Reading::Value(measure) => pick(*measure).ok_or_else(|| {
            let why = format!("{switch} read {measure}, not its unit");
            AdviseUnsupported::new(switch, Reason::from(why))
        }),
        Reading::Warming => Ok(warming),
        Reading::Unsupported { reason } => {
            Err(AdviseUnsupported::new(switch, reason.clone()))
        }
    }
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
    let eval = evaluate_once(gates, source, clock, request.interval);
    let wanted: SignalSet =
        [SignalId::Load, SignalId::Mem].into_iter().collect();
    let readings = sample_once(source, wanted, clock, request.interval);
    let read = |id: SignalId| {
        readings
            .iter()
            .find(|(read, _)| *read == id)
            .map_or(Reading::Warming, |(_, reading)| reading.clone())
    };
    let (load, mem) = (read(SignalId::Load), read(SignalId::Mem));
    advise_readings(request, &load, &mem, &eval.verdicts)
}

/// The exit code of an `advise` run: 0 for any advice, even `wait`;
/// 69 when something has no source here (src/advise:V6).
#[must_use]
pub const fn advise_exit_code(
    result: &Result<Advice, AdviseUnsupported>,
) -> u8 {
    match result {
        Ok(_) => 0,
        Err(_) => AdviseUnsupported::EXIT,
    }
}

#[cfg(test)]
mod tests;
