//! [`Trend`] and [`Trends`]: which way a signal moves, from its last two
//! values (src/signal:T7, V7).

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

use super::{Measure, Reading, SignalId};

/// Which way a signal moved between its last two values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trend {
    /// The last value is higher.
    Rising,
    /// The last value is lower.
    Falling,
    /// The last value is the same.
    Flat,
}

impl Trend {
    /// The trend from `previous` to `last`; `Flat` when they are equal or
    /// cannot be compared.
    #[must_use]
    pub fn between(previous: f64, last: f64) -> Self {
        match last.partial_cmp(&previous) {
            Some(Ordering::Greater) => Self::Rising,
            Some(Ordering::Less) => Self::Falling,
            Some(Ordering::Equal) | None => Self::Flat,
        }
    }
}

/// Renders as `--verbose` prints it: `rising`, `falling`, `flat`.
impl fmt::Display for Trend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Rising => "rising",
            Self::Falling => "falling",
            Self::Flat => "flat",
        })
    }
}

/// The last two values of every signal, for its [`Trend`].
///
/// Only values count: a [`Reading::Warming`] or [`Reading::Unsupported`]
/// round leaves the history as it was.
#[derive(Debug, Clone, Default)]
pub struct Trends {
    /// Per signal: the value before the last, if any, and the last.
    last: HashMap<SignalId, (Option<Measure>, Measure)>,
}

impl Trends {
    /// A history that has seen nothing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records the newest reading of `id`.
    pub fn record(&mut self, id: SignalId, reading: &Reading) {
        let Some(&value) = reading.measure() else {
            return;
        };
        let before = self.last.get(&id).map(|&(_, last)| last);
        self.last.insert(id, (before, value));
    }

    /// The trend of `id`, once it has two values.
    #[must_use]
    pub fn trend(&self, id: SignalId) -> Option<Trend> {
        let &(before, last) = self.last.get(&id)?;
        Some(Trend::between(before?.as_f64(), last.as_f64()))
    }
}

#[cfg(test)]
mod tests;
