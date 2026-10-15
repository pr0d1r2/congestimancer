use super::{cpu_fit, mem_fit};
use crate::units::{Bytes, Decimal};

const GIB: u64 = 1 << 30;

const fn dec(thousandths: u64) -> Decimal {
    Decimal::from_thousandths(thousandths)
}

#[test]
fn cpu_fit_is_cpus_times_limit_less_load() {
    assert_eq!(cpu_fit(8, dec(1_000), dec(5_000)), 3);
}

#[test]
fn cpu_fit_rounds_down() {
    assert_eq!(cpu_fit(8, dec(1_000), dec(4_200)), 3);
}

#[test]
fn cpu_fit_scales_with_the_load_limit() {
    assert_eq!(cpu_fit(4, dec(1_500), dec(1_000)), 5);
}

#[test]
fn cpu_fit_of_an_overloaded_machine_is_zero() {
    assert_eq!(cpu_fit(4, dec(1_000), dec(9_000)), 0);
}

#[test]
fn cpu_fit_never_overflows() {
    assert_eq!(cpu_fit(u64::MAX, dec(u64::MAX), dec(0)), u64::MAX / 1_000);
}

#[test]
fn mem_fit_divides_unreserved_memory_by_the_need() {
    assert_eq!(mem_fit(Bytes(10 * GIB), Bytes(2 * GIB), Bytes(GIB)), 8);
}

#[test]
fn mem_fit_rounds_down() {
    assert_eq!(mem_fit(Bytes(5 * GIB), Bytes(0), Bytes(2 * GIB)), 2);
}

#[test]
fn mem_fit_with_more_reserved_than_available_is_zero() {
    assert_eq!(mem_fit(Bytes(GIB), Bytes(3 * GIB), Bytes(GIB)), 0);
}

#[test]
fn mem_fit_of_a_zero_need_is_unbounded() {
    assert_eq!(mem_fit(Bytes(GIB), Bytes(0), Bytes(0)), u64::MAX);
}
