use super::UnitError;

const FORM: &str = "a whole number, like 4";

type Make = fn(String, &'static str) -> UnitError;

const MAKE: [Make; 4] = [
    |input, expected| UnitError::Malformed { input, expected },
    |input, expected| UnitError::Negative { input, expected },
    |input, expected| UnitError::OutOfRange { input, expected },
    |input, expected| UnitError::Overflow { input, expected },
];

fn each(input: &str) -> Vec<UnitError> {
    MAKE.iter().map(|m| m(input.to_owned(), FORM)).collect()
}

#[test]
fn every_variant_names_its_input_and_expected_form() {
    for e in each("4x") {
        assert_eq!(e.input(), "4x", "{e:?}");
        assert_eq!(e.expected(), FORM, "{e:?}");
    }
}

#[test]
fn the_message_quotes_the_input_and_the_form() {
    for e in each("4x") {
        let shown = e.to_string();
        assert!(shown.contains("`4x`"), "{shown}");
        assert!(shown.ends_with(&format!("expected {FORM}")), "{shown}");
    }
}

#[test]
fn the_message_says_what_was_wrong() {
    let says: Vec<String> = each("9").iter().map(ToString::to_string).collect();
    assert_eq!(
        says,
        [
            "`9` is not valid: expected a whole number, like 4",
            "`9` is negative: expected a whole number, like 4",
            "`9` is out of range: expected a whole number, like 4",
            "`9` is too large: expected a whole number, like 4",
        ]
    );
}
