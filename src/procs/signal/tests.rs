use super::{ProcMatch, ProcessTable, ProcsSignal};
use crate::procs::fixture::table;
use crate::procs::{Glob, Proc};
use crate::signal::{Measure, Sample, Signal, SignalId};
use crate::units::Count;

/// A fixture process table.
struct Fixture(&'static str);

impl ProcessTable for Fixture {
    fn processes(&mut self) -> Vec<Proc> {
        table(self.0)
    }
}

/// A signal over the `table` fixture, as cgmr pid 200.
fn over(by: ProcMatch) -> ProcsSignal<Fixture> {
    ProcsSignal::over(Fixture("table"), by, 200)
}

/// A count gauge.
const fn count(n: u64) -> Sample {
    Sample::Gauge(Measure::Count(Count(n)))
}

#[test]
fn max_procs_counts_by_name() {
    let mut signal = over(ProcMatch::Name(Glob::new("rustc")));
    assert_eq!(signal.sample(SignalId::MaxProcs), count(3));
}

#[test]
fn max_procs_leaves_out_cgmr_and_its_ancestors() {
    let mut signal = over(ProcMatch::Name(Glob::new("*")));
    assert_eq!(signal.sample(SignalId::MaxProcs), count(9));
}

#[test]
fn other_signals_are_unsupported() {
    let mut signal = over(ProcMatch::Name(Glob::new("*")));
    for id in [SignalId::Load, SignalId::MaxUserProcs] {
        let sample = signal.sample(id);
        assert!(
            matches!(&sample, Sample::Unsupported(r) if r.as_str().contains(id.name())),
            "{id}: {sample:?}"
        );
    }
}

#[test]
fn live_table_reads_a_count() {
    // The machine's processes; the value is never asserted (src:V4).
    let mut signal = ProcsSignal::live(ProcMatch::Name(Glob::new("*")));
    let sample = signal.sample(SignalId::MaxProcs);
    assert!(matches!(sample, Sample::Gauge(Measure::Count(_))));
}
