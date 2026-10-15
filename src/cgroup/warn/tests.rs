use super::load_warning;
use crate::cgroup::{Cgroup, Scope};
use crate::signal::{SignalId, SignalSet};

/// The gates `ids`.
fn gates(ids: &[SignalId]) -> SignalSet {
    let mut set = SignalSet::new();
    for &id in ids {
        set.insert(id);
    }
    set
}

#[test]
fn load_in_a_cgroup_warns_and_suggests_cpu_pressure() {
    let scope = Scope::Cgroup(Cgroup::new("/user.slice/job"));
    let warning = load_warning(&scope, gates(&[SignalId::Load]));
    assert_eq!(
        warning.as_deref(),
        Some(
            "warning: --load reads the host's load, which does not match \
             the CPUs of cgroup /user.slice/job; use --cpu-pressure instead"
        )
    );
}

#[test]
fn no_load_gate_no_warning() {
    let scope = Scope::Cgroup(Cgroup::new("/"));
    let warning = load_warning(&scope, gates(&[SignalId::CpuPressure]));
    assert_eq!(warning, None);
}

#[test]
fn load_on_the_host_is_fine() {
    let scope = Scope::Host {
        reason: "--cgroup host".into(),
    };
    assert_eq!(load_warning(&scope, gates(&[SignalId::Load])), None);
}
