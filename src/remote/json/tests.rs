use super::{Value, parse};

fn text(s: &str) -> Value {
    Value::Text(s.to_owned())
}

#[test]
fn scalars_parse() {
    assert_eq!(parse("null"), Ok(Value::Null));
    assert_eq!(parse(" true "), Ok(Value::Bool(true)));
    assert_eq!(parse("false"), Ok(Value::Bool(false)));
    assert_eq!(parse("-1.5e3"), Ok(Value::Number("-1.5e3".to_owned())));
    assert_eq!(parse("\"a\""), Ok(text("a")));
}

#[test]
fn nested_values_keep_their_order() {
    let parsed = parse("{\"b\":[1, {}], \"a\" : []}");
    let one = Value::Number("1".to_owned());
    let b = Value::List(vec![one, Value::Object(vec![])]);
    let want = vec![("b".to_owned(), b), ("a".to_owned(), Value::List(vec![]))];
    assert_eq!(parsed, Ok(Value::Object(want)));
}

#[test]
fn escapes_unescape() {
    let parsed = parse(r#""q\" b\\ s\/ \b\f\n\r\t \u00e9 \ud83d\ude00""#);
    assert_eq!(
        parsed,
        Ok(text("q\" b\\ s/ \u{8}\u{c}\n\r\t \u{e9} \u{1f600}"))
    );
}

#[test]
fn utf8_passes_through() {
    let word = "za\u{17c}\u{f3}\u{142}\u{107}";
    assert_eq!(parse(&format!("\"{word}\"")), Ok(text(word)));
}

fn all_refused(bad: &[&str]) {
    for input in bad {
        assert!(parse(input).is_err(), "{input:?}");
    }
}

#[test]
fn malformed_structure_is_an_error() {
    all_refused(&["", "{", "[1,]", "[1 2]", "{\"a\" 1}", "{1:2}", "1 2"]);
    all_refused(&["nul", "-", "1.2.3", "@"]);
}

#[test]
fn malformed_strings_are_an_error() {
    all_refused(&["\"open", "\"\\x\"", "\"\\u12\"", "\"a\nb\""]);
    all_refused(&["\"\\ud83d\"", "\"\\ud83dx\"", "\"\\ud83d\\u0041\""]);
}

#[test]
fn deep_nesting_is_refused() {
    let deep = format!("{}{}", "[".repeat(100), "]".repeat(100));
    assert!(parse(&deep).is_err());
    let shallow = format!("{}{}", "[".repeat(10), "]".repeat(10));
    assert!(parse(&shallow).is_ok());
}

#[test]
fn lookups_find_members_and_types() {
    let parsed = parse("{\"a\":\"x\",\"l\":[null]}").unwrap_or(Value::Null);
    assert_eq!(parsed.get("a").and_then(Value::as_text), Some("x"));
    assert_eq!(
        parsed.get("l").and_then(Value::as_list).map(<[_]>::len),
        Some(1)
    );
    assert_eq!(parsed.get("z"), None);
    assert_eq!(Value::Null.get("a"), None);
    assert_eq!(Value::Null.as_text(), None);
    assert_eq!(Value::Null.as_list(), None);
}
