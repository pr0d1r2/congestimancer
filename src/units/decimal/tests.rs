use super::{Decimal, parse_decimal};
use crate::units::UnitError;

fn thousandths(s: &str) -> Result<u64, UnitError> {
    parse_decimal(s).map(Decimal::thousandths)
}

#[test]
fn a_load_like_value_takes_decimals() {
    let good = [("0.5", 500), ("4", 4000), ("12.125", 12_125), ("0", 0)];
    for (s, t) in good {
        assert_eq!(thousandths(s), Ok(t), "{s}");
    }
}

#[test]
fn malformed_decimals_are_refused() {
    let bad = ["", "1.2345", "0,5", ".5", "5.", "1e3", "+1", "1x", "0.5 "];
    for s in bad {
        assert!(
            matches!(parse_decimal(s), Err(UnitError::Malformed { .. })),
            "{s}"
        );
    }
}

#[test]
fn a_negative_threshold_is_negative() {
    let e = parse_decimal("-0.5");
    assert!(matches!(e, Err(UnitError::Negative { .. })), "{e:?}");
}

#[test]
fn past_u64_thousandths_is_overflow() {
    let e = parse_decimal("18446744073709552");
    assert!(matches!(e, Err(UnitError::Overflow { .. })), "{e:?}");
}

#[test]
fn thousandths_and_floats_read_back() {
    let d = Decimal::from_thousandths(500);
    assert_eq!((d.thousandths(), d.as_f64()), (500, 0.5));
    assert_eq!("0.5".parse::<Decimal>(), Ok(d));
}
