//! The gates the resolved switches ask for: a [`Gate`] per gate switch,
//! one own-source gate per `--max-procs` value, and the custom gates
//! (src/gate, src/procs).

use std::path::PathBuf;

use super::SWITCHES;
use super::exit::Failure;
use super::layers::Resolved;
use super::table::{Adj, Role, SwitchSpec, Value};
use crate::gate::{Custom, Gate, Gates, Op};
use crate::procs::{ProcMatch, ProcsSignal, parse_proc_limit};
use crate::signal::{Measure, Sampled, SignalId, SignalSet};
use crate::units::{Bytes, Count, parse_clock_window};

/// What a gate needs besides its switch: the CPUs for per-CPU gates and
/// what running cgmr jobs reserved for `--needs-mem`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Fit {
    /// The logical CPUs.
    pub(crate) cpus: u64,
    /// Memory running cgmr jobs reserved.
    pub(crate) reserved: Bytes,
}

/// The gates and the shared signals they read.
#[derive(Debug, Default)]
pub(crate) struct Built {
    /// The gates, in table order.
    pub(crate) gates: Gates,
    /// The signals the shared-source gates read (src/signal:V3).
    pub(crate) signals: SignalSet,
}

/// The gates `resolved` asks for, leaving out the switches in `skip`
/// (`advise` sizes with `--load` and `--needs-mem` instead).
///
/// # Errors
///
/// A usage error for a value that does not parse.
pub(crate) fn build(
    resolved: &Resolved,
    fit: Fit,
    skip: &[&str],
) -> Result<Built, Failure> {
    let mut built = Built::default();
    let asked = SWITCHES
        .iter()
        .filter(|s| !skip.contains(&s.name()) && resolved.has(s.name()));
    for spec in asked {
        built.add(spec, resolved, fit)?;
    }
    Ok(built)
}

impl Built {
    /// Adds the gates switch `spec` asks for.
    fn add(
        &mut self,
        spec: &SwitchSpec,
        resolved: &Resolved,
        fit: Fit,
    ) -> Result<(), Failure> {
        match spec.role {
            Role::Gate(op, adj) => {
                let gate = signal_gate(spec, op, resolved)?;
                self.signals.insert(gate.signal);
                self.gates.push(adjusted(gate, adj, fit));
            }
            Role::Procs => self.procs(spec, resolved)?,
            Role::Custom => {
                self.gates.push_custom(custom(spec, resolved));
            }
            _ => {}
        }
        Ok(())
    }

    /// One gate with its own process count per value of `spec`
    /// (src/procs:V1, V2): at most N is strictly below N + 1.
    fn procs(
        &mut self,
        spec: &SwitchSpec,
        resolved: &Resolved,
    ) -> Result<(), Failure> {
        let id = signal(spec)?;
        for text in resolved.all(spec.name()) {
            let limit = parse_proc_limit(text).map_err(|e| {
                Failure::usage(format!("--{}: {e}", spec.name()))
            })?;
            let most = Count(limit.max.0.saturating_add(1));
            let by = match id {
                SignalId::MaxUserProcs => ProcMatch::User(limit.pattern),
                _ => ProcMatch::Name(limit.pattern),
            };
            let source = Sampled::new(ProcsSignal::live(by));
            self.gates
                .push_own(Gate::below(id, Measure::Count(most)), source);
        }
        Ok(())
    }
}

/// The signal switch `spec` gates on.
fn signal(spec: &SwitchSpec) -> Result<SignalId, Failure> {
    SignalId::for_switch(spec.name()).ok_or_else(|| {
        Failure::usage(format!("--{} reads no signal", spec.name()))
    })
}

/// The gate switch `spec` asks for, before any adjustment.
fn signal_gate(
    spec: &SwitchSpec,
    op: Op,
    resolved: &Resolved,
) -> Result<Gate, Failure> {
    let id = signal(spec)?;
    let name = spec.name();
    let missing = || Failure::usage(format!("--{name} needs a value"));
    match spec.value {
        Value::Flag => Ok(Gate::is(id, Measure::Flag(true))),
        Value::Window => {
            let window = resolved.parse(name, parse_clock_window)?;
            window.map(|w| Gate::within(id, w)).ok_or_else(missing)
        }
        value => {
            let measure = resolved.parse(name, |s| measure(value, s))?;
            measure.map(|m| compare(op, id, m)).ok_or_else(missing)
        }
    }
}

/// `s` as a `value` measure.
fn measure(value: Value, s: &str) -> Result<Measure, String> {
    value
        .measure(s)
        .unwrap_or_else(|| Err(format!("{s:?} is no threshold")))
}

/// The gate comparing `id` with `measure` as `op` says.
const fn compare(op: Op, id: SignalId, measure: Measure) -> Gate {
    match op {
        Op::Below => Gate::below(id, measure),
        Op::Above => Gate::above(id, measure),
        Op::Is => Gate::is(id, measure),
    }
}

/// `gate` with the adjustment its switch asks for.
const fn adjusted(gate: Gate, adj: Adj, fit: Fit) -> Gate {
    match adj {
        Adj::Whole => gate,
        Adj::PerCpu => gate.per_cpu(fit.cpus),
        Adj::Unreserved => gate.unreserved(fit.reserved),
    }
}

/// The custom gate switch `spec` asks for.
fn custom(spec: &SwitchSpec, resolved: &Resolved) -> Custom {
    let text = resolved.text(spec.name()).unwrap_or_default().to_owned();
    if spec.value == Value::Path {
        Custom::WhenFile(PathBuf::from(text))
    } else {
        Custom::When(text)
    }
}

#[cfg(test)]
mod tests;
