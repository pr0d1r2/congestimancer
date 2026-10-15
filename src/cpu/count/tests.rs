use std::io;
use std::num::NonZeroUsize;

use super::{cpu_count, cpus_from, load_per_cpu};
use crate::units::Decimal;

#[test]
fn cpu_count_is_at_least_one() {
    assert!(cpu_count() >= 1);
}

#[test]
fn cpus_are_what_available_parallelism_reads() {
    let eight = NonZeroUsize::new(8).ok_or(io::ErrorKind::InvalidInput);
    assert_eq!(cpus_from(eight.map_err(io::Error::from)), 8);
}

#[test]
fn cpus_are_one_when_the_count_cannot_be_read() {
    let failed = Err(io::Error::from(io::ErrorKind::Unsupported));
    assert_eq!(cpus_from(failed), 1);
}

#[test]
fn load_per_cpu_divides_by_cpus() {
    let load = Decimal::from_thousandths(6_000);
    assert_eq!(
        load_per_cpu(load, 4),
        Some(Decimal::from_thousandths(1_500))
    );
}

#[test]
fn load_per_cpu_rounds_down_to_thousandths() {
    let load = Decimal::from_thousandths(1_000);
    assert_eq!(load_per_cpu(load, 3), Some(Decimal::from_thousandths(333)));
}

#[test]
fn load_per_cpu_with_no_cpus_is_none() {
    assert_eq!(load_per_cpu(Decimal::from_thousandths(1), 0), None);
}
