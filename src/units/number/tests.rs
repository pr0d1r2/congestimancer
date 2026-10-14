use super::{LexError, lex, split};
use crate::units::UnitError;

const MAX: &str = "18446744073709551615";

#[test]
fn whole_numbers_lex_to_themselves() {
    for (s, n) in [("0", 0), ("42", 42), ("007", 7), (MAX, u64::MAX)] {
        assert_eq!(lex(s).map(|l| l.whole()), Ok(Some(n)), "{s}");
    }
}

#[test]
fn a_fraction_is_not_whole_but_trailing_zeros_are() {
    assert_eq!(lex("1.5").map(|l| l.whole()), Ok(None));
    assert_eq!(lex("3.000").map(|l| l.whole()), Ok(Some(3)));
    let long = format!("1.{}", "0".repeat(40));
    assert_eq!(lex(&long).map(|l| l.whole()), Ok(Some(1)));
}

#[test]
fn only_dot_separates_decimals_whatever_the_locale() {
    for s in ["1,5", "1 5", "1_5", "1'5", "1\u{66b}5"] {
        assert_eq!(lex(s), Err(LexError::Malformed), "{s}");
    }
}

#[test]
fn anything_but_digits_with_one_dot_is_malformed() {
    let bad = [
        "", ".", ".5", "5.", "1.2.3", "+1", " 1", "1 ", "1e3", "0x1", "inf",
        "NaN", "\u{661}", "-", "--1", "-.5", "1-",
    ];
    for s in bad {
        assert_eq!(lex(s), Err(LexError::Malformed), "{s:?}");
    }
}

#[test]
fn a_minus_before_a_number_is_negative() {
    for s in ["-1", "-0", "-1.5", &format!("-{MAX}0")] {
        assert_eq!(lex(s), Err(LexError::Negative), "{s}");
    }
}

#[test]
fn past_u64_is_overflow_never_wrapped() {
    assert_eq!(lex(&format!("{MAX}0")), Err(LexError::Overflow));
    assert_eq!(lex("18446744073709551616"), Err(LexError::Overflow));
}

#[test]
fn more_than_max_places_is_malformed() {
    assert!(lex("0.123456789012345678").is_ok());
    assert_eq!(lex("0.1234567890123456789"), Err(LexError::Malformed));
}

#[test]
fn times_floor_rounds_down_to_whole_units() {
    let floor = |s: &str, m: u64| lex(s).map(|l| l.times_floor(m));
    assert_eq!(floor("1.5", 1000), Ok(Some(1500)));
    assert_eq!(floor("0.0001", 1000), Ok(Some(0)));
    assert_eq!(floor("1.999", 10), Ok(Some(19)));
    assert_eq!(floor(&format!("{MAX}.9"), 1), Ok(Some(u64::MAX)));
}

#[test]
fn times_floor_past_u64_is_none() {
    let floor = |s: &str, m: u64| lex(s).map(|l| l.times_floor(m));
    assert_eq!(floor(MAX, 2), Ok(None));
    assert_eq!(floor("18446744073.709551616", 1_000_000_000), Ok(None));
}

#[test]
fn scaled_counts_in_tenths_hundredths_and_so_on() {
    let scaled = |s: &str, p: u32| lex(s).and_then(|l| l.scaled(p));
    assert_eq!(scaled("2.5", 1), Ok(25));
    assert_eq!(scaled("2.500", 1), Ok(25));
    assert_eq!(scaled("12.34", 2), Ok(1234));
    assert_eq!(scaled("7", 3), Ok(7000));
}

#[test]
fn scaled_refuses_more_places_and_overflow() {
    let scaled = |s: &str, p: u32| lex(s).and_then(|l| l.scaled(p));
    assert_eq!(scaled("2.55", 1), Err(LexError::Malformed));
    assert_eq!(scaled(MAX, 1), Err(LexError::Overflow));
}

#[test]
fn split_takes_the_leading_number() {
    assert_eq!(split("10GiB"), ("10", "GiB"));
    assert_eq!(split("1.5"), ("1.5", ""));
    assert_eq!(split("-3C"), ("-3", "C"));
    assert_eq!(split("GB"), ("", "GB"));
    assert_eq!(split("5\u{b0}C"), ("5", "\u{b0}C"));
}

#[test]
fn each_lex_error_becomes_its_unit_error() {
    let made = [LexError::Malformed, LexError::Negative, LexError::Overflow]
        .map(|e| e.error("x", "a form"));
    let kinds = made.each_ref().map(|e| match e {
        UnitError::Malformed { .. } => "malformed",
        UnitError::Negative { .. } => "negative",
        UnitError::Overflow { .. } => "overflow",
        _ => "other",
    });
    assert_eq!(kinds, ["malformed", "negative", "overflow"]);
    for e in made {
        assert_eq!((e.input(), e.expected()), ("x", "a form"));
    }
}
