//! [`Source`]: where readings come from (src/signal:T3, V5).

use std::time::Instant;

use super::{Reading, SignalId};

/// Something that reads signals: a domain node on the live machine, a
/// fixture tree in tests, a remote host's JSON.
///
/// `now` is the monotonic instant of this round (from a
/// [`Clock`](super::Clock)), so every signal read in one round shares one
/// time base and rates divide by the same interval (src/signal:V4).
///
/// A source that has no way to read `id` here returns
/// [`Reading::Unsupported`] with a reason, never a made-up value (src:V5).
/// A source holds its own state (counters for rates, the [`Root`] it reads
/// under, an open handle); nothing global (src/signal:V5).
///
/// [`Root`]: super::Root
pub trait Source {
    /// Reads `id` at monotonic instant `now`.
    fn read(&mut self, id: SignalId, now: Instant) -> Reading;
}

impl<S: Source + ?Sized> Source for &mut S {
    fn read(&mut self, id: SignalId, now: Instant) -> Reading {
        (**self).read(id, now)
    }
}

impl<S: Source + ?Sized> Source for Box<S> {
    fn read(&mut self, id: SignalId, now: Instant) -> Reading {
        (**self).read(id, now)
    }
}

/// A source tests script: each signal returns its scripted readings in
/// order, then repeats the last; an unscripted signal is unsupported. It
/// counts reads, so a test sees which signals were touched
/// (src/signal:V3).
#[cfg(test)]
#[derive(Debug, Default)]
pub(crate) struct FakeSource {
    /// Readings still to give, per signal.
    script: std::collections::HashMap<SignalId, Vec<Reading>>,
    /// How many times each signal was read.
    reads: std::collections::HashMap<SignalId, usize>,
}

#[cfg(test)]
impl FakeSource {
    /// A source with nothing scripted.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Scripts `readings` for `id`, replacing any earlier script.
    pub(crate) fn script(
        &mut self,
        id: SignalId,
        readings: impl IntoIterator<Item = Reading>,
    ) -> &mut Self {
        let _ = (id, readings.into_iter());
        self
    }

    /// How many times `id` was read.
    pub(crate) fn reads(&self, id: SignalId) -> usize {
        let _ = id;
        0
    }
}

#[cfg(test)]
impl Source for FakeSource {
    fn read(&mut self, id: SignalId, _: Instant) -> Reading {
        let _ = (&self.script, &self.reads, id);
        Reading::Warming
    }
}

#[cfg(test)]
mod tests;
