use super::{mem_per_cpu, mem_unreserved};
use crate::units::Bytes;

#[test]
fn mem_per_cpu_divides_available_by_cpus() {
    assert_eq!(mem_per_cpu(Bytes(8_000), 4), Some(Bytes(2_000)));
}

#[test]
fn mem_per_cpu_rounds_down() {
    assert_eq!(mem_per_cpu(Bytes(10), 3), Some(Bytes(3)));
}

#[test]
fn mem_per_cpu_with_no_cpus_is_none() {
    assert_eq!(mem_per_cpu(Bytes(10), 0), None);
}

#[test]
fn mem_unreserved_takes_the_reserved_total_off_available() {
    assert_eq!(mem_unreserved(Bytes(8_000), Bytes(3_000)), Bytes(5_000));
}

#[test]
fn mem_unreserved_never_drops_below_zero() {
    assert_eq!(mem_unreserved(Bytes(1_000), Bytes(3_000)), Bytes(0));
}
