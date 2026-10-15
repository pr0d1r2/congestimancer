use super::{Loads, parse_loadavg};
use crate::cpu::CpuError;
use crate::units::Decimal;

/// A decimal of `thousandths` thousandths.
const fn dec(thousandths: u64) -> Decimal {
    Decimal::from_thousandths(thousandths)
}

#[test]
fn loadavg_gives_the_three_averages() {
    assert_eq!(
        parse_loadavg("0.52 1.58 12.05 3/1234 5678\n"),
        Ok(Loads {
            one: dec(520),
            five: dec(1_580),
            fifteen: dec(12_050),
        })
    );
}

#[test]
fn loadavg_with_two_averages_is_malformed() {
    assert_eq!(
        parse_loadavg("0.52 1.58\n"),
        Err(CpuError::Malformed {
            file: "/proc/loadavg",
            line: "0.52 1.58".to_owned()
        })
    );
}

#[test]
fn loadavg_with_a_garbled_average_is_malformed() {
    assert_eq!(
        parse_loadavg("0.52 lots 1.0 1/2 3\n"),
        Err(CpuError::Malformed {
            file: "/proc/loadavg",
            line: "0.52 lots 1.0 1/2 3".to_owned()
        })
    );
}
