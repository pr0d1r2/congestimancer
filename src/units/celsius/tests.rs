use super::{Celsius, parse_celsius};
use crate::units::UnitError;

fn tenths(s: &str) -> Result<u32, UnitError> {
    parse_celsius(s).map(Celsius::tenths)
}

#[test]
fn a_temperature_is_a_number_and_c() {
    let good = [
        ("85C", 850),
        ("85c", 850),
        ("72.5C", 725),
        ("0C", 0),
        ("100.0C", 1000),
        ("429496729.5C", u32::MAX),
    ];
    for (s, t) in good {
        assert_eq!(tenths(s), Ok(t), "{s}");
    }
}

/// Not a temperature: no unit, another unit, a space, two decimals, a
/// comma, a degree sign.
const BAD: [&str; 11] = [
    "",
    "C",
    "85",
    "85F",
    "85K",
    "85 C",
    "85CC",
    "C85",
    "72.55C",
    "1,5C",
    "85\u{b0}C",
];

#[test]
fn malformed_temperatures_are_refused() {
    for s in BAD {
        assert!(
            matches!(parse_celsius(s), Err(UnitError::Malformed { .. })),
            "{s}"
        );
    }
}

#[test]
fn a_negative_threshold_is_negative() {
    let e = parse_celsius("-5C");
    assert!(matches!(e, Err(UnitError::Negative { .. })), "{e:?}");
}

#[test]
fn past_u32_tenths_is_overflow() {
    let e = parse_celsius("429496729.6C");
    assert!(matches!(e, Err(UnitError::Overflow { .. })), "{e:?}");
}

#[test]
fn the_error_names_the_input_and_the_form() {
    let Err(e) = parse_celsius("85F") else {
        unreachable!("85F parsed")
    };
    assert_eq!(e.input(), "85F");
    assert!(e.expected().contains("like 85C"), "{}", e.expected());
}

#[test]
fn tenths_and_floats_read_back() {
    let t = Celsius::from_tenths(725);
    assert_eq!((t.tenths(), t.as_f64()), (725, 72.5));
    assert_eq!("72.5C".parse::<Celsius>(), Ok(t));
}
