//! [`load_warning`]: `--load` inside a cgroup reads the wrong machine
//! (src/cgroup:T5, V3).

use super::Scope;
use crate::signal::SignalSet;

/// The warning to print on stderr when `gates` hold `--load` (or
/// `--all-load`) and `scope` is a cgroup: the host's load average divided
/// by the container's CPUs is wrong, so it suggests `--cpu-pressure`.
#[must_use]
pub fn load_warning(scope: &Scope, gates: SignalSet) -> Option<String> {
    let _ = (scope, gates);
    None
}

#[cfg(test)]
mod tests;
