use std::path::Path;
use std::time::Duration;

use super::{NetError, NetSignal, Os, selected};
use crate::signal::{
    Clock, ManualClock, Measure, Reading, Root, SignalId, Source,
};
use crate::sys::{InterfaceCounters, LinkCounters};
use crate::units::{Name, Rate, parse_name};

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
    let names: Vec<&str> = selected(&all, None)
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

/// The interface name `s`.
fn name(s: &str) -> Option<Name> {
    parse_name(s).ok()
}

/// A Linux net signal over fixture `case`, reading only `iface`.
fn linux_iface(case: &str, iface: &str) -> Option<NetSignal> {
    let net = NetSignal::on(root(case), Os::Linux);
    net.iface(name(iface)?).ok()
}

/// A rate of zero bytes per second.
const ZERO: Reading = Reading::Value(Measure::Rate(Rate(0)));

#[test]
fn net_iface_reads_that_interface_even_when_down() {
    let mut net = linux_iface("linux", "docker0");
    let readings = net.as_mut().map(|n| twice(n, SignalId::NetRx));
    assert_eq!(readings, Some([Reading::Warming, ZERO]));
}

#[test]
fn net_iface_counts_only_that_interface() {
    let all = [iface(("en0", true, false)), iface(("en1", true, false))];
    let one = name("en1");
    let names: Vec<&str> = selected(&all, one.as_ref())
        .iter()
        .map(|i| i.name.as_str())
        .collect();
    assert_eq!(names, ["en1"]);
}

#[test]
fn unknown_net_iface_is_a_usage_error() {
    let eth9 = name("eth9");
    let net = NetSignal::on(root("linux"), Os::Linux);
    let err = eth9.clone().and_then(|n| net.iface(n).err());
    assert_eq!(err, eth9.map(|name| NetError::UnknownIface { name }));
    let shown = err.map(|e| e.to_string()).unwrap_or_default();
    assert_eq!(shown, "--net-iface eth9: no such interface");
}

#[test]
fn net_iface_is_taken_when_interfaces_cannot_be_listed() {
    assert!(linux_iface("absent", "eth0").is_some());
}

#[test]
fn net_iface_that_went_away_is_unsupported() {
    let mut net = linux_iface("linux", "eth0");
    let first = net.as_mut().map(|net| {
        net.root = root("noflags");
        twice(net, SignalId::Net)[0].to_string()
    });
    assert_eq!(
        first.as_deref(),
        Some("unsupported: eth0: no such interface")
    );
}
