use std::time::Duration;

use super::parse_duration;
use crate::units::UnitError;

fn ms(s: &str) -> Result<u128, UnitError> {
    parse_duration(s).map(|d| d.as_millis())
}

#[test]
fn pairs_of_number_and_unit_add_up() {
    let good = [
        ("1h30m", 5_400_000),
        ("500ms", 500),
        ("2d", 172_800_000),
        ("1d2h3m4s5ms", 93_784_005),
        ("90s", 90_000),
        ("1m", 60_000),
        ("0s", 0),
        ("007s", 7000),
    ];
    for (s, n) in good {
        assert_eq!(ms(s), Ok(n), "{s}");
    }
}

#[test]
fn a_bare_number_or_a_stray_unit_is_malformed() {
    let bad = [
        "", "10", "h", "1.5h", "1H", "1 h", "1h 30m", "1w", "1y", "1us", "1hm",
        "1mss", "1h-30m", "1h30", "+1s", "1,5s",
    ];
    for s in bad {
        assert!(
            matches!(parse_duration(s), Err(UnitError::Malformed { .. })),
            "{s}"
        );
    }
}

#[test]
fn units_come_largest_first_and_once() {
    for s in ["30m1h", "1h1h", "1s1m", "1ms1s", "1m1m", "1s1ms1s"] {
        assert!(
            matches!(parse_duration(s), Err(UnitError::Malformed { .. })),
            "{s}"
        );
    }
}

#[test]
fn a_negative_duration_is_negative() {
    let e = parse_duration("-1s");
    assert!(matches!(e, Err(UnitError::Negative { .. })), "{e:?}");
}

#[test]
fn past_u64_milliseconds_is_overflow() {
    let max = parse_duration("18446744073709551615ms");
    assert_eq!(max, Ok(Duration::from_millis(u64::MAX)));
    let over = [
        "18446744073709551616ms",
        "213503982335d",
        "213503982334d23h",
        "99999999999999999999999s",
    ];
    for s in over {
        assert!(
            matches!(parse_duration(s), Err(UnitError::Overflow { .. })),
            "{s}"
        );
    }
}

#[test]
fn the_error_names_the_input_and_the_form() {
    let Err(e) = parse_duration("10") else {
        unreachable!("a bare number parsed")
    };
    assert_eq!(e.input(), "10");
    assert!(e.expected().contains("like 1h30m"), "{}", e.expected());
}
