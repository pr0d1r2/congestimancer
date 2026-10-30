//! [`SwitchSpec`] and its parts: what a switch takes and does. The rows
//! live in [`SWITCHES`](super::SWITCHES), one table that argv, the
//! environment, profiles, help, completions and the man page all read
//! (src/cli:V7).

use crate::gate::Op;

/// What a switch's value is, and so which `src/units` (or domain) parser
/// reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Value {
    /// No value: the switch is on or off.
    Flag,
    /// A number with decimals: load, run queue.
    Decimal,
    /// A share from 0 to 100.
    Percent,
    /// A size.
    Bytes,
    /// A size per second.
    Rate,
    /// A duration.
    Duration,
    /// A temperature.
    Celsius,
    /// A whole number of things.
    Count,
    /// A daily clock window, `22:00-06:00`.
    Window,
    /// A memory pressure level: `normal` or `warning`.
    Level,
    /// A load average window: `1m`, `5m` or `15m`.
    LoadWindow,
    /// A filesystem path.
    Path,
    /// Free text: a command, a host for `--rtt`.
    Text,
    /// A block device name (`--io-device`).
    Device,
    /// A network interface name (`--net-iface`).
    Iface,
    /// An ssh host (`--host`).
    Host,
    /// `auto`, `host` or a cgroup path.
    Cgroup,
    /// `PATTERN=N`.
    ProcLimit,
    /// `N/D`.
    StartRate,
    /// A safe name: a slot key, a profile.
    Name,
}

/// How a gate's reading is turned into the value it compares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Adj {
    /// As read.
    Whole,
    /// Per logical CPU.
    PerCpu,
    /// Less what running cgmr jobs reserved.
    Unreserved,
}

/// What a switch does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Role {
    /// A gate on a signal named like the switch.
    Gate(Op, Adj),
    /// A per-pattern process gate (`--max-procs`, `--max-user-procs`).
    Procs,
    /// A `--when` or `--when-file` gate.
    Custom,
    /// Picks what a gate reads: a path, device, interface, window.
    Select,
    /// Times the wait or holds a slot.
    Wait,
    /// Changes how the command runs.
    Run,
    /// Evaluates the gates on another host.
    Remote,
    /// Output and profiles.
    Meta,
}

/// One switch of the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct SwitchSpec {
    /// The long name without `--`.
    name: &'static str,
    /// What its value is.
    pub(crate) value: Value,
    /// What it does.
    pub(crate) role: Role,
    /// Whether it may be given more than once (src/cli:V9).
    repeats: bool,
    /// One line for `--help`.
    help: &'static str,
}

impl SwitchSpec {
    /// The long name without `--`: `io-busy`.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// The environment variable that sets it, `CGMR_IO_BUSY`; `None` for
    /// `--when`, which must never come from the environment
    /// (src/cli:V3).
    #[must_use]
    pub fn env(&self) -> Option<String> {
        (self.name != WHEN).then(|| env_name(self.name))
    }

    /// Whether it takes a value.
    #[must_use]
    pub fn takes_value(&self) -> bool {
        self.value != Value::Flag
    }

    /// Whether it may be given more than once.
    #[must_use]
    pub const fn repeats(&self) -> bool {
        self.repeats
    }

    /// Its one-line help.
    #[must_use]
    pub const fn help(&self) -> &'static str {
        self.help
    }
}

/// The switch that runs a command, and so has no environment default.
pub(crate) const WHEN: &str = "when";

/// The environment name of switch `name`: `CGMR_` and the name in upper
/// snake case (src/cli:I).
#[must_use]
pub fn env_name(name: &str) -> String {
    format!("CGMR_{}", name.replace('-', "_").to_ascii_uppercase())
}

/// The switch called `name` (without `--`).
pub(crate) fn spec(name: &str) -> Option<&'static SwitchSpec> {
    super::SWITCHES.iter().find(|s| s.name == name)
}

/// A switch given at most once.
pub(super) const fn one(
    name: &'static str,
    value: Value,
    role: Role,
    help: &'static str,
) -> SwitchSpec {
    SwitchSpec {
        name,
        value,
        role,
        repeats: false,
        help,
    }
}

/// A gate that passes below its threshold.
pub(super) const fn below(
    name: &'static str,
    value: Value,
    help: &'static str,
) -> SwitchSpec {
    one(name, value, Role::Gate(Op::Below, Adj::Whole), help)
}

/// A gate that passes above its threshold.
pub(super) const fn above(
    name: &'static str,
    value: Value,
    help: &'static str,
) -> SwitchSpec {
    one(name, value, Role::Gate(Op::Above, Adj::Whole), help)
}

/// A gate on a state.
pub(super) const fn state(
    name: &'static str,
    value: Value,
    help: &'static str,
) -> SwitchSpec {
    one(name, value, Role::Gate(Op::Is, Adj::Whole), help)
}

/// A switch of `role` with no value.
pub(super) const fn flag(
    name: &'static str,
    role: Role,
    help: &'static str,
) -> SwitchSpec {
    one(name, Value::Flag, role, help)
}

/// A switch of `role` that takes a name and may repeat.
pub(super) const fn names(
    name: &'static str,
    role: Role,
    help: &'static str,
) -> SwitchSpec {
    SwitchSpec {
        repeats: true,
        ..one(name, Value::Name, role, help)
    }
}

/// A per-pattern process gate, which repeats.
pub(super) const fn procs(
    name: &'static str,
    help: &'static str,
) -> SwitchSpec {
    SwitchSpec {
        repeats: true,
        ..one(name, Value::ProcLimit, Role::Procs, help)
    }
}

#[cfg(test)]
mod tests;
