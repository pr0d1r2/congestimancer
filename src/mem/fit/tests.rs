use super::mem_per_cpu;
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
