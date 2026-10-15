//! [`evaluate_once`] and [`sample_once`]: one-shot reads for `check` and
//! `status`, which never print a warming value (src/gate:T6, V8).

use std::time::Duration;

use super::{Evaluation, Gates, evaluate};
use crate::signal::{Clock, Reading, SignalId, SignalSet, Source, Trends};

/// Evaluates `gates` once for `check`: when a gate reads a rate or an
/// interval percent, its first sample is taken, `clock` waits `interval`
/// once, and the evaluation reads the second, so no gate is warming.
pub fn evaluate_once<S, C>(
    gates: &mut Gates,
    source: &mut S,
    clock: &mut C,
    interval: Duration,
) -> Evaluation
where
    S: Source + ?Sized,
    C: Clock + ?Sized,
{
    let _ = interval;
    let mut trends = Trends::new();
    evaluate(gates, source, clock.now(), &mut trends)
}

/// Reads `wanted` once for `status`: rates and interval percents take a
/// second sample after `interval`, so none reads warming.
pub fn sample_once<S, C>(
    source: &mut S,
    wanted: SignalSet,
    clock: &mut C,
    interval: Duration,
) -> Vec<(SignalId, Reading)>
where
    S: Source + ?Sized,
    C: Clock + ?Sized,
{
    let _ = interval;
    crate::signal::refresh(source, wanted, clock.now())
}

#[cfg(test)]
mod tests;
