use std::path::Path;
use std::time::Duration;

use super::{NetSignal, Os, up_interfaces};
use crate::signal::{
    Clock, ManualClock, Measure, Reading, Root, SignalId, Source,
};
use crate::sys::{InterfaceCounters, LinkCounters};
use crate::units::Rate;

/// The fixture tree `tests/fixtures/net/<case>`, laid out like `/`.
fn root(case: &str) -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/net").join(case))
}

/// Reads `id` twice, a second apart.
fn twice(net: &mut NetSignal, id: SignalId) -> [Reading; 2] {
    let mut clock = ManualClock::new();
    let first = net.read(id, clock.now());
    clock.advance(Duration::from_secs(1));
    [first, net.read(id, clock.now())]
}

/// An interface: its name, whether up, whether loopback.
fn iface((name, up, loopback): (&str, bool, bool)) -> InterfaceCounters {
    InterfaceCounters {
        name: name.to_owned(),
        up,
        loopback,
        rx: LinkCounters::default(),
        tx: LinkCounters::default(),
    }
}

#[test]
fn rates_count_up_non_loopback_interfaces() {
    let all = [
        iface(("lo0", true, true)),
        iface(("en0", true, false)),
        iface(("en1", false, false)),
        iface(("utun0", true, false)),
    ];
    let names: Vec<&str> = up_interfaces(&all)
        .iter()
        .map(|i| i.name.as_str())
        .collect();
    assert_eq!(names, ["en0", "utun0"]);
}

#[test]
fn linux_rates_warm_up_then_read_bytes_per_second() {
    for id in [SignalId::NetRx, SignalId::NetTx, SignalId::Net] {
        let mut net = NetSignal::on(root("linux"), Os::Linux);
        let zero = Reading::Value(Measure::Rate(Rate(0)));
        assert_eq!(twice(&mut net, id), [Reading::Warming, zero], "{id}");
    }
}

#[test]
fn linux_without_proc_net_dev_is_unsupported() {
    let mut net = NetSignal::on(root("absent"), Os::Linux);
    let [first, _] = twice(&mut net, SignalId::Net);
    let why = first.to_string();
    assert!(why.contains("/proc/net/dev"), "{why}");
}

#[test]
fn a_torn_proc_net_dev_is_unsupported_naming_the_line() {
    let mut net = NetSignal::on(root("torn"), Os::Linux);
    let [first, _] = twice(&mut net, SignalId::NetRx);
    let why = first.to_string();
    assert!(why.contains("line 3"), "{why}");
}

#[test]
fn macos_rates_warm_up_then_read_a_rate_or_are_unsupported() {
    let mut net = NetSignal::on(Root::system(), Os::Macos);
    let [first, second] = twice(&mut net, SignalId::Net);
    if cfg!(target_os = "macos") {
        assert_eq!(first, Reading::Warming);
        assert!(
            matches!(
                second,
                Reading::Value(Measure::Rate(_)) | Reading::Warming
            ),
            "{second}"
        );
    } else {
        assert!(matches!(first, Reading::Unsupported { .. }), "{first}");
    }
}

#[test]
fn other_signals_are_unsupported() {
    let mut net = NetSignal::new(root("linux"));
    let [first, _] = twice(&mut net, SignalId::Disk);
    assert!(matches!(first, Reading::Unsupported { .. }), "{first}");
}
