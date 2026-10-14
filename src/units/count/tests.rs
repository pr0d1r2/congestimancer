use super::{Count, parse_count};
use crate::units::UnitError;

#[test]
fn a_count_is_digits_alone() {
    let good = [
        ("0", 0),
        ("4", 4),
        ("007", 7),
        ("18446744073709551615", u64::MAX),
    ];
    for (s, n) in good {
        assert_eq!(parse_count(s), Ok(Count(n)), "{s}");
    }
}

#[test]
fn a_count_is_a_whole_number() {
    let bad = [
        "1.5", "4.0", "4.", ".4", "", "4 ", "4k", "1e3", "+4", "0x10",
    ];
    for s in bad {
        assert!(
            matches!(parse_count(s), Err(UnitError::Malformed { .. })),
            "{s}"
        );
    }
}

#[test]
fn a_negative_count_is_negative() {
    let e = parse_count("-1");
    assert!(matches!(e, Err(UnitError::Negative { .. })), "{e:?}");
}

#[test]
fn past_u64_is_overflow() {
    let e = parse_count("18446744073709551616");
    assert!(matches!(e, Err(UnitError::Overflow { .. })), "{e:?}");
}

#[test]
fn the_error_names_the_input_and_the_form() {
    let Err(e) = parse_count("four") else {
        unreachable!("four parsed")
    };
    assert_eq!(
        (e.input(), e.expected()),
        ("four", "a whole number, like 4")
    );
    assert_eq!("12".parse::<Count>(), Ok(Count(12)));
}
