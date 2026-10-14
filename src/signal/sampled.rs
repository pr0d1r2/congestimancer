//! [`Signal`] and [`Sampled`]: a domain node says what it sees now, and
//! the sampler turns counters into rates (src/signal:T4).

use std::time::Instant;

use super::{Counter, Measure, Reading, Reason, Sampler, SignalId, Source};

/// What a domain node sees now for one signal, before any interval math.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Sample {
    /// A value that stands alone: free memory, load, temperature.
    Gauge(Measure),
    /// A running total that needs a previous sample to mean anything.
    Counter(Counter),
    /// No source for this signal here, and why.
    Unsupported(Reason),
}

/// A domain node's raw view of its signals: no clock, no history.
///
/// Implement this to read a domain (`src/cpu`, `src/io`, `src/net`, ...)
/// and wrap it in [`Sampled`] to get a [`Source`]: gauges pass through,
/// counters go through a [`Sampler`], so no domain repeats the warm-up and
/// reset rules (src/signal:V1, V2).
pub trait Signal {
    /// What `id` looks like now.
    fn sample(&mut self, id: SignalId) -> Sample;
}

/// A [`Signal`] with a [`Sampler`]: a [`Source`].
#[derive(Debug, Default)]
pub struct Sampled<S> {
    /// The domain that samples.
    signal: S,
    /// The bases of its counters.
    sampler: Sampler,
}

impl<S> Sampled<S> {
    /// Wraps `signal` with a sampler that has seen nothing.
    pub fn new(signal: S) -> Self {
        Self {
            signal,
            sampler: Sampler::new(),
        }
    }

    /// The wrapped signal.
    pub const fn get_ref(&self) -> &S {
        &self.signal
    }

    /// The wrapped signal, mutably.
    pub const fn get_mut(&mut self) -> &mut S {
        &mut self.signal
    }
}

impl<S: Signal> Source for Sampled<S> {
    fn read(&mut self, id: SignalId, now: Instant) -> Reading {
        let _ = (&mut self.sampler, id, now);
        Reading::Warming
    }
}

#[cfg(test)]
mod tests;
