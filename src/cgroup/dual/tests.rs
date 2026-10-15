use std::path::Path;
use std::time::Instant;

use super::Dual;
use crate::cgroup::{Cgroup, Scope, Scoped};
use crate::signal::{FakeSource, Measure, Reading, Root, SignalId, SignalSet};
use crate::units::{Bytes, Percent};

/// The fixture tree `tests/fixtures/cgroup/limited`, laid out like `/`.
fn root() -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/cgroup/limited"))
}

/// A reading of `hundredths` percent.
fn pct(hundredths: u16) -> Reading {
    let p = Percent::from_hundredths(hundredths).unwrap_or_default();
    Reading::Value(Measure::Percent(p))
}

/// A host with 50% CPU pressure and 9 GiB available.
fn host() -> FakeSource {
    let mut host = FakeSource::new();
    host.script(SignalId::CpuPressure, [pct(5000)]);
    host.script(
        SignalId::Mem,
        [Reading::Value(Measure::Bytes(Bytes(9 << 30)))],
    );
    host
}

/// The gates `ids`.
fn gates(ids: &[SignalId]) -> SignalSet {
    let mut set = SignalSet::new();
    for &id in ids {
        set.insert(id);
    }
    set
}

#[test]
fn a_cgroup_shows_its_values_beside_the_hosts() {
    let scope = Scope::Cgroup(Cgroup::new("/user.slice/job"));
    let mut scoped = Scoped::new(root(), scope, host());
    let ids = gates(&[SignalId::Load, SignalId::CpuPressure, SignalId::Mem]);
    let rows = scoped.dual(ids, Instant::now());
    let gib = |n: u64| Reading::Value(Measure::Bytes(Bytes(n << 30)));
    assert_eq!(
        rows,
        [
            Dual::new(SignalId::CpuPressure, pct(1250), pct(5000)),
            Dual::new(SignalId::Mem, gib(5), gib(9)),
        ]
    );
}

#[test]
fn on_the_host_the_cgroup_column_says_why_it_is_empty() {
    let scope = Scope::Host {
        reason: "--cgroup host".into(),
    };
    let mut scoped = Scoped::new(root(), scope, host());
    let rows = scoped.dual(gates(&[SignalId::CpuPressure]), Instant::now());
    let none = Reading::unsupported("no cgroup: --cgroup host");
    assert_eq!(rows, [Dual::new(SignalId::CpuPressure, none, pct(5000))]);
}

#[test]
fn a_row_renders_both_values() {
    let row = Dual::new(SignalId::CpuPressure, pct(1250), pct(5000));
    assert_eq!(row.to_string(), "cpu-pressure: cgroup 12.5% | host 50%");
}
