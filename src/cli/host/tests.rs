use std::time::Instant;

use super::{HostSource, Selectors, domain, machine};
use crate::cli::exit::USAGE;
use crate::cli::layers::{Layer, Resolved};
use crate::signal::{Reading, Root, SignalId, SignalSet, Source};

const FIXTURE: &str = "tests/fixtures/cpu/linux";

fn host(resolved: &Resolved, wanted: &[SignalId]) -> Option<HostSource> {
    let selectors = Selectors::new(resolved, Root::new(FIXTURE)).ok()?;
    let wanted: SignalSet = wanted.iter().copied().collect();
    HostSource::new(resolved, &selectors, wanted).ok()
}

#[test]
fn every_shared_signal_has_a_domain() {
    let own = [
        SignalId::MaxProcs,
        SignalId::MaxUserProcs,
        SignalId::CpuThrottle,
    ];
    for id in SignalId::ALL {
        assert_eq!(domain(id).is_none(), own.contains(&id), "{id}");
    }
}

#[test]
fn a_signal_not_asked_for_has_no_source() {
    let source = host(&Resolved::default(), &[SignalId::Load]);
    let read = source.map(|mut s| s.read(SignalId::Gpu, Instant::now()));
    let reason = match read {
        Some(Reading::Unsupported { reason }) => reason.to_string(),
        _ => String::new(),
    };
    assert_eq!(reason, "--gpu has no source here");
}

#[test]
fn every_domain_builds() {
    let rtt =
        Resolved::default().with("rtt-host", &["localhost:1"], Layer::Flag);
    let all: Vec<SignalId> = SignalId::ALL.to_vec();
    assert!(host(&rtt, &all).is_some());
}

#[test]
fn rtt_without_a_host_is_a_usage_error() {
    let selectors = Selectors::new(&Resolved::default(), Root::new(FIXTURE));
    let wanted: SignalSet = [SignalId::Rtt].into_iter().collect();
    let built = selectors
        .and_then(|s| HostSource::new(&Resolved::default(), &s, wanted));
    assert_eq!(built.err().map(|f| f.code), Some(USAGE));
}

#[test]
fn selectors_are_parsed() {
    let window = Resolved::default().with("load-window", &["5m"], Layer::Flag);
    assert!(host(&window, &[SignalId::Load]).is_some());
    let device = Resolved::default().with("io-device", &["sda"], Layer::Flag);
    assert!(host(&device, &[SignalId::IoRead]).is_some());
    let bad = Resolved::default().with("load-window", &["2m"], Layer::Flag);
    assert!(host(&bad, &[SignalId::Load]).is_none());
}

#[test]
fn machine_warns_about_load_in_a_cgroup_only() {
    let mut warnings = Vec::new();
    let wanted: SignalSet = [SignalId::Load].into_iter().collect();
    let hosted = Resolved::default().with("cgroup", &["host"], Layer::Flag);
    let built = machine(&hosted, &Root::new(FIXTURE), wanted, &mut |w| {
        warnings.push(w.to_owned());
    });
    assert!(built.is_ok());
    assert!(warnings.is_empty());
}
