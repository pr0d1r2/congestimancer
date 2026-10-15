use std::path::Path;
use std::time::Duration;

use super::CgroupSignal;
use crate::cgroup::Cgroup;
use crate::signal::{
    Clock, Counter, ManualClock, Measure, Reading, Root, Sample, Sampled,
    Signal, SignalId, Source,
};
use crate::units::{Bytes, Percent};

/// The fixture tree `tests/fixtures/cgroup/<case>`, laid out like `/`.
fn root(case: &str) -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/cgroup").join(case))
}

/// The job cgroup of fixture `limited`.
fn job() -> CgroupSignal {
    CgroupSignal::new(root("limited"), Cgroup::new("/user.slice/job"))
}

/// The session scope of fixture `unlimited`.
fn session() -> CgroupSignal {
    let scope = Cgroup::new("/user.slice/session-1.scope");
    CgroupSignal::new(root("unlimited"), scope)
}

/// The reason of an unsupported sample, if it is one.
fn reason(sample: Sample) -> Option<String> {
    match sample {
        Sample::Unsupported(reason) => Some(reason.as_str().to_owned()),
        Sample::Gauge(_) | Sample::Counter(_) => None,
    }
}

#[test]
fn throttling_is_time_busy_stopped() {
    let busy = Duration::from_micros(1_234_567);
    assert_eq!(
        job().sample(SignalId::CpuThrottle),
        Sample::Counter(Counter::Busy { busy })
    );
}

#[test]
fn no_cpu_controller_is_unsupported() {
    assert_eq!(
        reason(session().sample(SignalId::CpuThrottle)),
        Some(
            "/user.slice/session-1.scope has no cpu controller to throttle it"
                .to_owned()
        )
    );
}

#[test]
fn a_missing_cpu_stat_is_unsupported_naming_it() {
    let mut gone = CgroupSignal::new(root("limited"), Cgroup::new("/gone"));
    assert_eq!(
        reason(gone.sample(SignalId::CpuThrottle)),
        Some("/sys/fs/cgroup/gone/cpu.stat: not present".to_owned())
    );
}

#[test]
fn throttling_reads_as_a_share_of_the_interval() {
    let mut source = Sampled::new(job());
    let mut clock = ManualClock::new();
    let first = source.read(SignalId::CpuThrottle, clock.now());
    clock.advance(Duration::from_secs(1));
    let second = source.read(SignalId::CpuThrottle, clock.now());
    let none = Percent::from_hundredths(0).map(Measure::Percent);
    assert_eq!((first, second.measure().copied()), (Reading::Warming, none));
}

#[test]
fn signals_it_does_not_own_are_unsupported() {
    let why = reason(job().sample(SignalId::Load));
    assert_eq!(why, Some("load is not a cgroup signal".to_owned()));
}

/// A gauge of `hundredths` percent.
fn pct(hundredths: u16) -> Sample {
    let p = Percent::from_hundredths(hundredths).unwrap_or_default();
    Sample::Gauge(Measure::Percent(p))
}

#[test]
fn pressure_gates_read_the_cgroup_files() {
    let mut job = job();
    let got = [
        SignalId::CpuPressure,
        SignalId::MemPressure,
        SignalId::MemPressureFull,
        SignalId::IoPressure,
        SignalId::IoPressureFull,
    ]
    .map(|id| job.sample(id));
    assert_eq!(got, [pct(1250), pct(75), pct(25), pct(500), pct(400)]);
}

#[test]
fn missing_pressure_is_unsupported_naming_the_file() {
    assert_eq!(
        reason(session().sample(SignalId::IoPressure)),
        Some(
            "/sys/fs/cgroup/user.slice/session-1.scope/io.pressure: \
             not present"
                .to_owned()
        )
    );
}

#[test]
fn a_missing_full_line_is_unsupported() {
    let mut container = CgroupSignal::new(root("container"), Cgroup::new("/"));
    assert_eq!(
        reason(container.sample(SignalId::CpuPressure)),
        None,
        "cpu.pressure has a some line"
    );
    assert_eq!(
        reason(container.sample(SignalId::IoPressureFull)),
        Some("/sys/fs/cgroup/io.pressure has no full line".to_owned())
    );
}

#[test]
fn memory_left_is_the_tightest_limit_in_the_lineage() {
    let left = job().sample(SignalId::Mem);
    assert_eq!(left, Sample::Gauge(Measure::Bytes(Bytes(5 << 30))));
}

#[test]
fn no_memory_limit_is_unsupported() {
    assert_eq!(
        reason(session().sample(SignalId::Mem)),
        Some("/user.slice/session-1.scope has no memory limit".to_owned())
    );
}
