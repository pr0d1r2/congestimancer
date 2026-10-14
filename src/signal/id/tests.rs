use std::collections::BTreeSet;

use super::SignalId;

/// The human doc every signal is named in (.:V18).
const CONGESTION: &str = include_str!("../../../docs/CONGESTION.md");

#[test]
fn every_signal_has_its_own_name() {
    let names: BTreeSet<&str> =
        SignalId::ALL.iter().map(|id| id.name()).collect();
    assert_eq!(names.len(), SignalId::ALL.len());
    assert!(!names.contains(""));
}

#[test]
fn every_name_is_a_switch_in_the_congestion_doc() {
    for id in SignalId::ALL {
        let switch = format!("`--{}", id.name());
        assert!(CONGESTION.contains(&switch), "{switch}");
    }
}

#[test]
fn a_name_reads_back_as_its_signal() {
    for id in SignalId::ALL {
        assert_eq!(SignalId::for_switch(id.name()), Some(id), "{id:?}");
        assert_eq!(id.to_string(), id.name());
    }
}

#[test]
fn switches_on_one_quantity_share_its_signal() {
    let shared = [
        ("all-load", SignalId::Load),
        ("all-run-queue", SignalId::RunQueue),
        ("all-mem", SignalId::Mem),
        ("needs-mem", SignalId::Mem),
    ];
    for (switch, id) in shared {
        assert_eq!(SignalId::for_switch(switch), Some(id), "{switch}");
    }
}

/// Switches that select, steer or run commands: none reads a signal.
const NOT_SIGNALS: [&str; 17] = [
    "",
    "--load",
    "Load",
    "load-window",
    "disk-path",
    "io-device",
    "net-iface",
    "rtt-host",
    "gpu-device",
    "cgroup",
    "when",
    "when-file",
    "interval",
    "timeout",
    "host",
    "slots",
    "profile",
];

#[test]
fn selectors_and_strategies_are_not_signals() {
    for switch in NOT_SIGNALS {
        assert_eq!(SignalId::for_switch(switch), None, "{switch}");
    }
}

/// The signals measured over an interval: rates and interval percents.
const DELTA: [SignalId; 12] = [
    SignalId::CpuIdle,
    SignalId::Steal,
    SignalId::SwapRate,
    SignalId::IoRead,
    SignalId::IoWrite,
    SignalId::Io,
    SignalId::IoBusy,
    SignalId::IoLatency,
    SignalId::NetRx,
    SignalId::NetTx,
    SignalId::Net,
    SignalId::CpuThrottle,
];

#[test]
fn rates_and_interval_percents_need_two_samples() {
    for id in SignalId::ALL {
        assert_eq!(id.is_delta(), DELTA.contains(&id), "{id:?}");
    }
}
