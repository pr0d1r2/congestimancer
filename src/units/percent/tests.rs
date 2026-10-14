use super::{Percent, parse_percent};
use crate::units::UnitError;

fn hundredths(s: &str) -> Result<u16, UnitError> {
    parse_percent(s).map(Percent::hundredths)
}

#[test]
fn a_percent_is_a_number_and_an_optional_sign() {
    let good = [
        ("80", 8000),
        ("80%", 8000),
        ("12.5%", 1250),
        ("0.01", 1),
        ("0", 0),
        ("100%", 10_000),
        ("100.00", 10_000),
    ];
    for (s, h) in good {
        assert_eq!(hundredths(s), Ok(h), "{s}");
    }
}

#[test]
fn past_100_is_out_of_range() {
    for s in ["100.01", "101%", "1000", "99999999999999"] {
        assert!(
            matches!(parse_percent(s), Err(UnitError::OutOfRange { .. })),
            "{s}"
        );
    }
}

#[test]
fn malformed_percents_are_refused() {
    let bad = [
        "", "%", "80%%", "80 %", "%80", "12.345", "1,5", "80p", "0x10",
    ];
    for s in bad {
        assert!(
            matches!(parse_percent(s), Err(UnitError::Malformed { .. })),
            "{s}"
        );
    }
}

#[test]
fn a_negative_percent_is_negative() {
    let e = parse_percent("-5%");
    assert!(matches!(e, Err(UnitError::Negative { .. })), "{e:?}");
}

#[test]
fn the_error_names_the_input_and_the_form() {
    let Err(e) = parse_percent("150%") else {
        unreachable!("150% parsed")
    };
    assert_eq!(e.input(), "150%");
    assert!(e.expected().contains("0 to 100"), "{}", e.expected());
}

#[test]
fn hundredths_round_trip_up_to_full() {
    assert_eq!(
        Percent::from_hundredths(1250).map(Percent::hundredths),
        Some(1250)
    );
    assert_eq!(Percent::from_hundredths(10_000), Some(Percent::FULL));
    assert_eq!(Percent::from_hundredths(10_001), None);
}

#[test]
fn as_f64_reads_in_percent() {
    let half = Percent::from_hundredths(1250).map(Percent::as_f64);
    assert_eq!(half, Some(12.5));
    assert_eq!("80%".parse::<Percent>().map(Percent::as_f64), Ok(80.0));
}
