//! [`Gate`] and [`Verdict`]: one threshold against one reading, compared
//! strictly (src/gate:T1, V1).

use std::cmp::Ordering;
use std::fmt;

use crate::cpu::load_per_cpu;
use crate::mem::{level_within, mem_per_cpu, mem_unreserved};
use crate::signal::{Measure, Reading, Reason, SignalId, Trend};
use crate::units::{Bytes, ClockWindow};

/// How a reading must stand against its threshold for the gate to pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    /// Load-like: the reading must be strictly below the threshold.
    Below,
    /// Capacity-like: the reading must be strictly above the threshold.
    Above,
    /// A state: a flag equal to the threshold, a level at most the
    /// threshold (`--mem-level`), a time inside the window (`--between`).
    Is,
}

/// What a gate compares its reading with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Threshold {
    /// A value in the reading's unit.
    Value(Measure),
    /// A daily window a time of day must fall in (`--between`).
    Window(ClockWindow),
}

/// Renders the value in its unit, or the window as `22:00-06:00`.
impl fmt::Display for Threshold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(measure) => measure.fmt(f),
            Self::Window(w) => write!(f, "{}-{}", w.start(), w.end()),
        }
    }
}

/// How the raw reading is turned into the value the gate compares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Adjust {
    /// The reading as it is (`--all-load`, `--all-mem`, every other gate).
    Whole,
    /// The reading per logical CPU, `N` given (`--load`, `--run-queue`,
    /// `--mem`), from [`cpu_count`](crate::cpu::cpu_count).
    PerCpu(u64),
    /// Available memory less what running cgmr jobs reserved
    /// (`--needs-mem`), the total from
    /// [`reserved_total`](crate::slot::reserved_total).
    Unreserved(Bytes),
}

/// One gate: a signal, how it must stand, and against what.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Gate {
    /// The signal the gate reads.
    pub signal: SignalId,
    /// How the reading must stand against the threshold.
    pub op: Op,
    /// What the reading is compared with.
    pub threshold: Threshold,
    /// How the raw reading is turned into the compared value.
    pub adjust: Adjust,
}

impl Gate {
    /// Passes while `signal` reads strictly below `threshold`.
    #[must_use]
    pub const fn below(signal: SignalId, threshold: Measure) -> Self {
        Self::new(signal, Op::Below, Threshold::Value(threshold))
    }

    /// Passes while `signal` reads strictly above `threshold`.
    #[must_use]
    pub const fn above(signal: SignalId, threshold: Measure) -> Self {
        Self::new(signal, Op::Above, Threshold::Value(threshold))
    }

    /// Passes while `signal` reads `state`: the same flag, or a level at
    /// most `state` (src/mem:V5).
    #[must_use]
    pub const fn is(signal: SignalId, state: Measure) -> Self {
        Self::new(signal, Op::Is, Threshold::Value(state))
    }

    /// Passes while `signal`, a time of day, falls inside `window`.
    #[must_use]
    pub const fn within(signal: SignalId, window: ClockWindow) -> Self {
        Self::new(signal, Op::Is, Threshold::Window(window))
    }

    /// A gate with no adjustment.
    const fn new(signal: SignalId, op: Op, threshold: Threshold) -> Self {
        Self {
            signal,
            op,
            threshold,
            adjust: Adjust::Whole,
        }
    }

    /// The same gate, comparing the reading per CPU of `cpus`.
    #[must_use]
    pub const fn per_cpu(mut self, cpus: u64) -> Self {
        self.adjust = Adjust::PerCpu(cpus);
        self
    }

    /// The same gate, comparing the reading less `reserved` bytes.
    #[must_use]
    pub const fn unreserved(mut self, reserved: Bytes) -> Self {
        self.adjust = Adjust::Unreserved(reserved);
        self
    }

    /// The verdict on `reading`; `trend` rides along on a failure for
    /// `--verbose`.
    #[must_use]
    pub fn judge(&self, reading: &Reading, trend: Option<Trend>) -> Verdict {
        match reading {
            Reading::Value(raw) => self.judge_value(*raw, trend),
            Reading::Warming => Verdict::Warming,
            Reading::Unsupported { reason } => Verdict::Unsupported {
                reason: reason.clone(),
            },
        }
    }

    /// The verdict on a value, adjusted then compared.
    fn judge_value(&self, raw: Measure, trend: Option<Trend>) -> Verdict {
        let Some(value) = self.adjust.apply(raw) else {
            return self.mismatch(raw);
        };
        match self.passes(value) {
            Some(true) => Verdict::Pass,
            Some(false) => Verdict::Fail {
                value,
                threshold: self.threshold,
                trend,
            },
            None => self.mismatch(value),
        }
    }

    /// Whether `value` passes; `None` when it cannot be compared with the
    /// threshold at all.
    fn passes(&self, value: Measure) -> Option<bool> {
        match (self.op, self.threshold) {
            (Op::Below, Threshold::Value(t)) => {
                order(value, t).map(Ordering::is_lt)
            }
            (Op::Above, Threshold::Value(t)) => {
                order(value, t).map(Ordering::is_gt)
            }
            (Op::Is, Threshold::Value(t)) => state(value, t),
            (Op::Is, Threshold::Window(w)) => match value {
                Measure::Clock(t) => Some(w.contains(t)),
                _ => None,
            },
            (Op::Below | Op::Above, Threshold::Window(_)) => None,
        }
    }

    /// The verdict on a value the gate cannot compare: unsupported, never
    /// a silent pass (src/gate:V3).
    fn mismatch(&self, value: Measure) -> Verdict {
        let why = format!(
            "{} read {value}, which cannot be compared with {}",
            self.signal, self.threshold
        );
        Verdict::Unsupported {
            reason: Reason::from(why),
        }
    }
}

impl Adjust {
    /// The compared value from the raw one; `None` when the adjustment
    /// does not fit the unit or divides by no CPUs.
    fn apply(self, raw: Measure) -> Option<Measure> {
        match (self, raw) {
            (Self::Whole, _) => Some(raw),
            (Self::PerCpu(cpus), Measure::Number(n)) => {
                load_per_cpu(n, cpus).map(Measure::Number)
            }
            (Self::PerCpu(cpus), Measure::Bytes(b)) => {
                mem_per_cpu(b, cpus).map(Measure::Bytes)
            }
            (Self::Unreserved(reserved), Measure::Bytes(b)) => {
                Some(Measure::Bytes(mem_unreserved(b, reserved)))
            }
            (Self::PerCpu(_) | Self::Unreserved(_), _) => None,
        }
    }
}

/// How `a` orders against `b` when both are quantities in one unit.
fn order(a: Measure, b: Measure) -> Option<Ordering> {
    use Measure as M;
    match (a, b) {
        (M::Number(a), M::Number(b)) => Some(a.cmp(&b)),
        (M::Count(a), M::Count(b)) => Some(a.cmp(&b)),
        (M::Percent(a), M::Percent(b)) => Some(a.cmp(&b)),
        (M::Bytes(a), M::Bytes(b)) => Some(a.cmp(&b)),
        (M::Rate(a), M::Rate(b)) => Some(a.cmp(&b)),
        (M::Duration(a), M::Duration(b)) => Some(a.cmp(&b)),
        (M::Celsius(a), M::Celsius(b)) => Some(a.cmp(&b)),
        _ => None,
    }
}

/// Whether state `value` meets `wanted`: the same flag, or a level at
/// most the limit (src/mem:V5).
fn state(value: Measure, wanted: Measure) -> Option<bool> {
    match (value, wanted) {
        (Measure::Flag(a), Measure::Flag(b)) => Some(a == b),
        (Measure::Level(a), Measure::Level(b)) => Some(level_within(a, b)),
        _ => None,
    }
}

/// What one gate says this round.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Verdict {
    /// The system is strictly better than the threshold.
    Pass,
    /// It is not: the compared value, the threshold, and which way the
    /// signal moves when known.
    Fail {
        /// The value compared, after any per-CPU or reserved adjustment.
        value: Measure,
        /// The threshold it missed.
        threshold: Threshold,
        /// Which way the signal moves, once it has two values.
        trend: Option<Trend>,
    },
    /// No value yet; ask again next round.
    Warming,
    /// No source here, or a reading the gate cannot compare: exit 69.
    Unsupported {
        /// Why, for the user.
        reason: Reason,
    },
}

impl Verdict {
    /// Whether the gate passed.
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }
}

#[cfg(test)]
mod tests;
