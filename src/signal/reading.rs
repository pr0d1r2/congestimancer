//! [`Reading`]: what a signal says now, carrying its unit so a report
//! renders it without knowing the source (src/signal:T2, V6).

use std::borrow::Cow;
use std::fmt;
use std::time::Duration;

use crate::units::{
    Bytes, Celsius, ClockTime, Count, Decimal, Percent, Rate, render_duration,
};

/// What one signal reads now.
///
/// A gate passes or fails on a [`Reading::Value`]; it waits on
/// [`Reading::Warming`]; and it reports [`Reading::Unsupported`] as
/// "unsupported on this OS" (exit 69, src:V5), never as a pass.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Reading {
    /// A value with its unit.
    Value(Measure),
    /// A rate or interval percent on its first sample, or after its
    /// counter went backwards: no value yet, never a fake zero
    /// (src/signal:V1, V2).
    Warming,
    /// This platform, machine or configuration has no source for the
    /// signal; `reason` says why, for the user.
    Unsupported {
        /// Why there is no source, such as `no PSI on macOS`.
        reason: Reason,
    },
}

impl Reading {
    /// An [`Reading::Unsupported`] reading with its reason.
    #[must_use]
    pub fn unsupported(reason: impl Into<Cow<'static, str>>) -> Self {
        Self::Unsupported {
            reason: Reason(reason.into()),
        }
    }

    /// The value, if the reading has one.
    #[must_use]
    pub const fn measure(&self) -> Option<&Measure> {
        match self {
            Self::Value(measure) => Some(measure),
            Self::Warming | Self::Unsupported { .. } => None,
        }
    }
}

/// Renders the value in its unit, `warming up`, or `unsupported: <reason>`.
impl fmt::Display for Reading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(measure) => measure.fmt(f),
            Self::Warming => f.write_str("warming up"),
            Self::Unsupported { reason } => write!(f, "unsupported: {reason}"),
        }
    }
}

/// Why a signal has no source here, worded for the user.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Reason(Cow<'static, str>);

impl Reason {
    /// The reason as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<T: Into<Cow<'static, str>>> From<T> for Reason {
    fn from(reason: T) -> Self {
        Self(reason.into())
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A value and its unit, in the types `src/units` parses thresholds into,
/// so a gate compares like with like and a report renders each in the form
/// its switch takes (`12.5%`, `1GiB`, `50MB/s`, `72.5C`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Measure {
    /// A plain number with decimals: load, run queue (`1.5`).
    Number(Decimal),
    /// A whole number of things: processes, requests in flight.
    Count(Count),
    /// A share from 0 to 100.
    Percent(Percent),
    /// A size: free memory, free disk.
    Bytes(Bytes),
    /// A size per second: IO, network, swap.
    Rate(Rate),
    /// A time: IO latency, round trip, user idle.
    Duration(Duration),
    /// A temperature.
    Celsius(Celsius),
    /// A yes or no: cool, on AC, low power off.
    Flag(bool),
    /// A memory pressure level.
    Level(Level),
    /// A local time of day.
    Clock(ClockTime),
}

impl Measure {
    /// The value as a plain number, for trends and comparisons: seconds
    /// for a duration, `0` or `1` for a flag, the rank for a level,
    /// minutes since midnight for a time of day.
    #[must_use]
    pub fn as_f64(self) -> f64 {
        match self {
            Self::Number(n) => n.as_f64(),
            Self::Count(Count(n))
            | Self::Bytes(Bytes(n))
            | Self::Rate(Rate(n)) => n as f64,
            Self::Percent(p) => p.as_f64(),
            Self::Duration(d) => d.as_secs_f64(),
            Self::Celsius(c) => c.as_f64(),
            Self::Flag(on) => f64::from(u8::from(on)),
            Self::Level(level) => f64::from(level as u8),
            Self::Clock(t) => {
                f64::from(t.hour()) * 60.0 + f64::from(t.minute())
            }
        }
    }
}

/// Renders in the form the matching switch takes: `12.5%`, `1KiB`,
/// `50MB/s`, `1s500ms`, `72.5C`, `yes`, `warning`, `22:00`.
impl fmt::Display for Measure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => n.fmt(f),
            Self::Count(n) => n.fmt(f),
            Self::Percent(p) => p.fmt(f),
            Self::Bytes(b) => b.fmt(f),
            Self::Rate(r) => r.fmt(f),
            Self::Duration(d) => f.write_str(&render_duration(*d)),
            Self::Celsius(c) => c.fmt(f),
            Self::Flag(on) => f.write_str(if *on { "yes" } else { "no" }),
            Self::Level(level) => level.fmt(f),
            Self::Clock(t) => t.fmt(f),
        }
    }
}

/// A memory pressure level, from calm to critical (`--mem-level`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// No pressure.
    Normal,
    /// The system is reclaiming memory.
    Warning,
    /// The system is short of memory.
    Critical,
}

/// Renders as the `--mem-level` value: `normal`, `warning`, `critical`.
impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Normal => "normal",
            Self::Warning => "warning",
            Self::Critical => "critical",
        })
    }
}

#[cfg(test)]
mod tests;
