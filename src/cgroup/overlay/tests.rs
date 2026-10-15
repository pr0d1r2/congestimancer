use std::path::Path;
use std::time::Instant;

use super::{CGROUP_SIGNALS, Scoped};
use crate::cgroup::{Cgroup, Scope};
use crate::signal::{FakeSource, Measure, Reading, Root, SignalId, Source};
use crate::units::{Bytes, Decimal, Percent};

/// The fixture tree `tests/fixtures/cgroup/<case>`, laid out like `/`.
fn root(case: &str) -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/cgroup").join(case))
}

/// A reading of `n` GiB.
fn gib(n: u64) -> Reading {
    Reading::Value(Measure::Bytes(Bytes(n << 30)))
}

/// A reading of `hundredths` percent.
fn pct(hundredths: u16) -> Reading {
    let p = Percent::from_hundredths(hundredths).unwrap_or_default();
    Reading::Value(Measure::Percent(p))
}

/// A host with load 4, 9 GiB available, and 50% of every pressure.
fn host(mem: Reading) -> FakeSource {
    let mut host = FakeSource::new();
    let load = Measure::Number(Decimal::from_thousandths(4000));
    host.script(SignalId::Load, [Reading::Value(load)]);
    host.script(SignalId::Mem, [mem]);
    for id in CGROUP_SIGNALS {
        if id != SignalId::Mem {
            host.script(id, [pct(5000)]);
        }
    }
    host
}

/// The job cgroup of fixture `limited` over `host`.
fn in_job(host: FakeSource) -> Scoped<FakeSource> {
    let scope = Scope::Cgroup(Cgroup::new("/user.slice/job"));
    Scoped::new(root("limited"), scope, host)
}

/// Host values only, for `reason`, over `host`.
fn on_host(reason: &'static str, host: FakeSource) -> Scoped<FakeSource> {
    let scope = Scope::Host {
        reason: reason.into(),
    };
    Scoped::new(root("limited"), scope, host)
}

#[test]
fn in_a_cgroup_pressure_comes_from_its_files() {
    let mut scoped = in_job(host(gib(9)));
    let now = Instant::now();
    let read = [SignalId::CpuPressure, SignalId::IoPressureFull]
        .map(|id| scoped.read(id, now));
    assert_eq!(read, [pct(1250), pct(400)]);
}

#[test]
fn in_a_cgroup_other_signals_come_from_the_host() {
    let mut scoped = in_job(host(gib(9)));
    let load = scoped.read(SignalId::Load, Instant::now());
    assert_eq!(load.to_string(), "4");
    assert_eq!(scoped.host_mut().reads(SignalId::CpuPressure), 0);
}

#[test]
fn in_a_cgroup_memory_is_the_lesser_of_cgroup_and_host() {
    let now = Instant::now();
    assert_eq!(in_job(host(gib(9))).read(SignalId::Mem, now), gib(5));
    assert_eq!(in_job(host(gib(2))).read(SignalId::Mem, now), gib(2));
    let warming = in_job(host(Reading::Warming)).read(SignalId::Mem, now);
    assert_eq!(warming, gib(5));
}

#[test]
fn a_cgroup_without_a_memory_limit_reads_host_memory() {
    let scope = Scope::Cgroup(Cgroup::new("/user.slice/session-1.scope"));
    let mut scoped = Scoped::new(root("unlimited"), scope, host(gib(9)));
    assert_eq!(scoped.read(SignalId::Mem, Instant::now()), gib(9));
}

#[test]
fn on_the_host_every_signal_comes_from_the_host() {
    let mut scoped = on_host("--cgroup host", host(gib(9)));
    let now = Instant::now();
    assert_eq!(scoped.read(SignalId::CpuPressure, now), pct(5000));
    assert_eq!(scoped.read(SignalId::Mem, now), gib(9));
}

#[test]
fn on_the_host_cpu_throttle_is_unsupported_and_says_why() {
    let mut scoped = on_host("cgroup v1 is unsupported", host(gib(9)));
    assert_eq!(
        scoped.read(SignalId::CpuThrottle, Instant::now()),
        Reading::unsupported(
            "--cpu-throttle needs a cgroup v2: cgroup v1 is unsupported"
        )
    );
}

#[test]
fn in_a_cgroup_cpu_throttle_warms_up() {
    let mut scoped = in_job(host(gib(9)));
    let first = scoped.read(SignalId::CpuThrottle, Instant::now());
    assert_eq!(first, Reading::Warming);
}

#[test]
fn it_keeps_its_scope() {
    let scoped = on_host("--cgroup host", host(gib(9)));
    assert_eq!(scoped.scope().cgroup(), None);
}
