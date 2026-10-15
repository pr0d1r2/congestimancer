use super::{Arity, Switch, SwitchFault, SwitchTable, TestTable, parse_switch};
use crate::profile::{At, ProfileError};

/// Line 7 of `p/.cgmr`.
fn at() -> At {
    At::new("p/.cgmr", 7)
}

/// A switch line split as `(flag, value)`, or why it is no switch.
type Split = Result<(String, Option<String>), &'static str>;

/// The switch on a line, as `(flag, value)`.
fn split(line: &str) -> Split {
    parse_switch(line, at())
        .map(|s| (s.flag().to_owned(), s.value().map(str::to_owned)))
}

/// `(flag, value)` from literals.
fn pair(flag: &str, value: Option<&str>) -> (String, Option<String>) {
    (flag.to_owned(), value.map(str::to_owned))
}

#[test]
fn a_switch_splits_at_a_space_or_an_equals_sign() {
    assert_eq!(split("--load 2"), Ok(pair("--load", Some("2"))));
    assert_eq!(split("--load=2"), Ok(pair("--load", Some("2"))));
    assert_eq!(split("--load \t 2"), Ok(pair("--load", Some("2"))));
    assert_eq!(split("-l 2"), Ok(pair("-l", Some("2"))));
    assert_eq!(split("--quiet"), Ok(pair("--quiet", None)));
    assert_eq!(split("--when=a=b c"), Ok(pair("--when", Some("a=b c"))));
}

#[test]
fn one_pair_of_matching_quotes_is_dropped() {
    assert_eq!(split("--path \"a b\""), Ok(pair("--path", Some("a b"))));
    assert_eq!(split("--path 'a b'"), Ok(pair("--path", Some("a b"))));
    assert_eq!(split("--path 'a\""), Ok(pair("--path", Some("'a\""))));
    assert_eq!(split("--path '"), Ok(pair("--path", Some("'"))));
    assert_eq!(split("--path ''"), Ok(pair("--path", Some(""))));
}

#[test]
fn dashes_alone_are_no_switch() {
    assert!(split("--").is_err());
    assert!(split("- 2").is_err());
    assert!(split("--=2").is_err());
}

#[test]
fn a_switch_keeps_its_line_and_renders_as_argv() {
    let switch = parse_switch("--load 2", at());
    assert_eq!(switch.as_ref().map(Switch::at), Ok(&at()));
    assert_eq!(switch.as_ref().map(Switch::args), Ok(vec!["--load", "2"]));
}

/// Checks a switch line against the test table.
fn check(line: &str) -> Result<Arity, ProfileError> {
    parse_switch(line, at())
        .map_err(|_| ProfileError::Untrusted { at: at() })
        .and_then(|s| s.check(&TestTable))
}

#[test]
fn the_cli_table_accepts_its_switches() {
    assert_eq!(check("--load 2"), Ok(Arity::Single));
    assert_eq!(check("--path x"), Ok(Arity::Repeatable));
    assert_eq!(check("--quiet"), Ok(Arity::Single));
}

#[test]
fn an_unknown_switch_names_file_and_line() {
    let e = check("--loud 2");
    assert_eq!(
        e,
        Err(ProfileError::UnknownSwitch {
            at: at(),
            flag: "--loud".to_owned(),
        })
    );
    let text = e.err().map(|e| e.to_string()).unwrap_or_default();
    assert_eq!(text, "p/.cgmr:7: unknown switch --loud");
}

#[test]
fn a_bad_value_meets_the_same_unit_rules_as_the_cli() {
    let e = check("--mem 4XB").err();
    assert!(
        matches!(&e, Some(ProfileError::BadValue { flag, .. }) if flag == "--mem"),
        "{e:?}"
    );
    let text = e.map(|e| e.to_string()).unwrap_or_default();
    assert!(text.starts_with("p/.cgmr:7: --mem: `4XB`"), "{text}");
}

#[test]
fn values_the_cli_refuses_are_refused() {
    assert!(check("--load -1").is_err());
    assert!(check("--quiet yes").is_err());
    assert!(check("--path").is_err());
}

#[test]
fn the_test_table_says_why() {
    let table = TestTable;
    assert_eq!(table.check("--x", None), Err(SwitchFault::Unknown));
    assert!(matches!(
        table.check("--when", None),
        Err(SwitchFault::BadValue(_))
    ));
    assert_eq!(table.check("--when", Some("true")), Ok(Arity::Single));
    assert_eq!(table.check("--mem", Some("1GB")), Ok(Arity::Single));
}
