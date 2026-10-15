use std::time::Duration;

use super::Rates;
use crate::signal::{Clock, ManualClock, Measure, Reading, SignalId};
use crate::sys::{InterfaceCounters, LinkCounters};
use crate::units::Rate;

/// An up, non-loopback interface with `rx` and `tx` bytes so far.
fn iface(name: &str, rx: u64, tx: u64) -> InterfaceCounters {
    let link = |bytes| LinkCounters {
        bytes,
        ..LinkCounters::default()
    };
    InterfaceCounters {
        name: name.to_owned(),
        up: true,
        loopback: false,
        rx: link(rx),
        tx: link(tx),
    }
}

/// Reads `id` once per round of interfaces, a second apart.
fn run(id: SignalId, rounds: &[Vec<InterfaceCounters>]) -> Vec<Reading> {
    let mut clock = ManualClock::new();
    let mut rates = Rates::default();
    rounds
        .iter()
        .map(|round| {
            let chosen: Vec<&InterfaceCounters> = round.iter().collect();
            let reading = rates.read(id, &chosen, clock.now());
            clock.advance(Duration::from_secs(1));
            reading
        })
        .collect()
}

/// A rate reading.
const fn rate(bytes_per_second: u64) -> Reading {
    Reading::Value(Measure::Rate(Rate(bytes_per_second)))
}

/// Two interfaces, then each a second later.
fn two_rounds() -> [Vec<InterfaceCounters>; 2] {
    [
        vec![iface("en0", 100, 10), iface("en1", 1000, 0)],
        vec![iface("en0", 300, 40), iface("en1", 1500, 5)],
    ]
}

#[test]
fn rx_is_received_bytes_per_second_over_the_interfaces() {
    let readings = run(SignalId::NetRx, &two_rounds());
    assert_eq!(readings, [Reading::Warming, rate(700)]);
}

#[test]
fn tx_is_sent_bytes_per_second_over_the_interfaces() {
    let readings = run(SignalId::NetTx, &two_rounds());
    assert_eq!(readings, [Reading::Warming, rate(35)]);
}

#[test]
fn net_is_received_plus_sent() {
    let readings = run(SignalId::Net, &two_rounds());
    assert_eq!(readings, [Reading::Warming, rate(735)]);
}

#[test]
fn a_wrapped_interface_warms_up_even_when_the_sum_grew() {
    let rounds = [
        vec![iface("en0", 4_294_967_290, 0), iface("en1", 0, 0)],
        vec![iface("en0", 10, 0), iface("en1", 1 << 40, 0)],
        vec![iface("en0", 20, 0), iface("en1", 1 << 40, 0)],
    ];
    let readings = run(SignalId::NetRx, &rounds);
    assert_eq!(readings, [Reading::Warming, Reading::Warming, rate(10)]);
}

#[test]
fn an_interface_that_came_up_warms_up_instead_of_a_jump() {
    let rounds = [
        vec![iface("en0", 100, 0)],
        vec![iface("en0", 200, 0), iface("utun3", 1 << 40, 0)],
        vec![iface("en0", 300, 0), iface("utun3", 1 << 40, 0)],
    ];
    let readings = run(SignalId::NetRx, &rounds);
    assert_eq!(readings, [Reading::Warming, Reading::Warming, rate(100)]);
}

#[test]
fn an_interface_that_went_away_warms_up() {
    let rounds = [
        vec![iface("en0", 100, 0), iface("en1", 1 << 40, 0)],
        vec![iface("en0", 200, 0)],
    ];
    let readings = run(SignalId::NetRx, &rounds);
    assert_eq!(readings, [Reading::Warming, Reading::Warming]);
}

#[test]
fn no_interface_reads_zero() {
    let readings = run(SignalId::Net, &[vec![], vec![]]);
    assert_eq!(readings, [Reading::Warming, rate(0)]);
}

#[test]
fn signals_keep_their_own_bases() {
    let mut clock = ManualClock::new();
    let mut rates = Rates::default();
    let [a, b] = two_rounds();
    let a: Vec<&InterfaceCounters> = a.iter().collect();
    let b: Vec<&InterfaceCounters> = b.iter().collect();
    rates.read(SignalId::NetRx, &a, clock.now());
    rates.read(SignalId::NetTx, &a, clock.now());
    clock.advance(Duration::from_secs(1));
    assert_eq!(rates.read(SignalId::NetRx, &b, clock.now()), rate(700));
    assert_eq!(rates.read(SignalId::NetTx, &b, clock.now()), rate(35));
}

#[test]
fn forget_warms_a_signal_up_again() {
    let mut clock = ManualClock::new();
    let mut rates = Rates::default();
    let [a, b] = two_rounds();
    let a: Vec<&InterfaceCounters> = a.iter().collect();
    let b: Vec<&InterfaceCounters> = b.iter().collect();
    rates.read(SignalId::Net, &a, clock.now());
    rates.forget(SignalId::Net);
    clock.advance(Duration::from_secs(1));
    assert_eq!(rates.read(SignalId::Net, &b, clock.now()), Reading::Warming);
}
