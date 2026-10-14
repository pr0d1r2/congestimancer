use super::{MAX_LEN, Name, parse_name};
use crate::units::UnitError;

#[test]
fn safe_identifiers_are_names() {
    let longest = "a".repeat(MAX_LEN);
    let good = [
        "a",
        "build-1",
        "my_profile.v2",
        "...",
        ".hidden",
        "A9",
        &longest,
    ];
    for s in good {
        assert_eq!(parse_name(s).as_ref().map(Name::as_str), Ok(s), "{s}");
    }
}

/// Unsafe as a file name, a path segment or an argv word.
const BAD: [&str; 14] = [
    "", "-x", "--help", ".", "..", "a/b", "a b", "a\0b", "\u{e9}", "a:b",
    "a\\b", "$x", "a\nb", "*",
];

#[test]
fn unsafe_identifiers_are_malformed() {
    for s in BAD {
        assert!(
            matches!(parse_name(s), Err(UnitError::Malformed { .. })),
            "{s:?}"
        );
    }
}

#[test]
fn past_64_characters_is_out_of_range() {
    let long = "a".repeat(MAX_LEN + 1);
    let e = parse_name(&long);
    assert!(matches!(e, Err(UnitError::OutOfRange { .. })), "{e:?}");
}

#[test]
fn the_error_names_the_input_and_the_form() {
    let Err(e) = parse_name("a/b") else {
        unreachable!("a/b parsed")
    };
    assert_eq!(e.input(), "a/b");
    assert!(e.expected().contains("1 to 64"), "{}", e.expected());
    assert_eq!("x".parse::<Name>().as_ref().map(Name::as_str), Ok("x"));
}
