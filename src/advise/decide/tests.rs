use std::time::Instant;

use super::{Action, Advice, Limit, Request, decide};
use crate::gate::{Checked, Gate, Gates, evaluate};
use crate::signal::{FakeSource, Measure, Reading, SignalId, Trends};
use crate::units::{Bytes, Count, Decimal};

const GIB: u64 = 1 << 30;

const fn dec(thousandths: u64) -> Decimal {
    Decimal::from_thousandths(thousandths)
}

/// The verdict of an `--fds` gate below `limit` reading `open` files.
fn fds_gate(open: u64, limit: u64) -> Vec<Checked> {
    let mut source = FakeSource::new();
    let reading = Reading::Value(Measure::Count(Count(open)));
    source.script(SignalId::Fds, [reading]);
    let mut gates = Gates::new();
    gates.push(Gate::below(SignalId::Fds, Measure::Count(Count(limit))));
    evaluate(&mut gates, &mut source, Instant::now(), &mut Trends::new())
        .verdicts
}

/// 8 CPUs at load 5 (cpu fit 3), 10GiB available (mem fit 10 by 1GiB).
fn advise(request: Request, gates: &[Checked]) -> Advice {
    decide(&request, dec(5_000), Bytes(10 * GIB), gates)
}

#[test]
fn a_request_defaults_to_load_one_and_a_gib_per_job() {
    let request = Request::new(8);
    assert_eq!(request.load_limit, dec(1_000));
    assert_eq!(request.needs_mem, Bytes(GIB));
    assert_eq!(request.jobs, None);
    assert_eq!(request.reserved, Bytes(0));
}

#[test]
fn jobs_are_the_smaller_fit_and_name_it() {
    let advice = advise(Request::new(8), &[]);
    assert_eq!(advice.jobs, 3);
    assert_eq!(advice.limit, Limit::Cpu);
    assert_eq!(advice.action, Action::Run);
    assert!(advice.blocking.is_empty());
}

#[test]
fn memory_bounds_when_it_fits_fewer() {
    let request = Request::new(8).with_reserved(Bytes(8 * GIB));
    let advice = advise(request, &[]);
    assert_eq!((advice.jobs, advice.limit), (2, Limit::Mem));
}

#[test]
fn the_need_per_job_sizes_the_memory_fit() {
    let request = Request::new(8).with_needs_mem(Bytes(5 * GIB));
    assert_eq!(advise(request, &[]).limit, Limit::Mem);
}

#[test]
fn jobs_asked_for_and_fitting_run_bounded_by_jobs() {
    let advice = advise(Request::new(8).with_jobs(2), &[]);
    assert_eq!(advice.jobs, 2);
    assert_eq!(advice.limit, Limit::Jobs);
    assert_eq!(advice.action, Action::Run);
}

#[test]
fn jobs_asked_for_equal_to_the_fit_run() {
    let advice = advise(Request::new(8).with_jobs(3), &[]);
    assert_eq!((advice.jobs, advice.action), (3, Action::Run));
}

#[test]
fn more_jobs_than_fit_reduce() {
    let advice = advise(Request::new(8).with_jobs(6), &[]);
    assert_eq!(advice.jobs, 3);
    assert_eq!(advice.limit, Limit::Cpu);
    assert_eq!(advice.action, Action::Reduce);
}

#[test]
fn nothing_fitting_waits() {
    let advice = decide(&Request::new(4), dec(9_000), Bytes(10 * GIB), &[]);
    assert_eq!(advice.jobs, 0);
    assert_eq!(advice.limit, Limit::Cpu);
    assert_eq!(advice.action, Action::Wait);
}

#[test]
fn zero_jobs_asked_for_waits_on_jobs() {
    let advice = advise(Request::new(8).with_jobs(0), &[]);
    assert_eq!((advice.jobs, advice.limit), (0, Limit::Jobs));
    assert_eq!(advice.action, Action::Wait);
}

#[test]
fn a_passing_gate_changes_nothing() {
    let advice = advise(Request::new(8), &fds_gate(10, 100));
    assert_eq!((advice.jobs, advice.action), (3, Action::Run));
    assert!(advice.blocking.is_empty());
}

#[test]
fn a_failing_gate_waits_and_names_its_switch() {
    let gates = fds_gate(200, 100);
    let advice = advise(Request::new(8).with_jobs(2), &gates);
    assert_eq!(advice.jobs, 0);
    assert_eq!(advice.limit, Limit::Gate("fds".to_owned()));
    assert_eq!(advice.action, Action::Wait);
    assert_eq!(advice.blocking, gates);
}

#[test]
fn limits_and_actions_render_as_words() {
    let words: Vec<String> = [Limit::Cpu, Limit::Mem, Limit::Jobs]
        .iter()
        .map(ToString::to_string)
        .chain([Limit::Gate("io-busy".to_owned()).to_string()])
        .chain(
            [Action::Run, Action::Reduce, Action::Wait]
                .iter()
                .map(ToString::to_string),
        )
        .collect();
    assert_eq!(
        words,
        ["cpu", "mem", "jobs", "io-busy", "run", "reduce", "wait"]
    );
}
