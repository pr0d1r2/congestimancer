use super::{PropValue, Props, parse_props, render_props};

fn table() -> Props {
    Props::from([
        ("Bytes (Read)".to_owned(), PropValue::Int(5_888_208_695_296)),
        ("Is Charging".to_owned(), PropValue::Bool(false)),
        (
            "Power Source State".to_owned(),
            PropValue::Text("AC Power".into()),
        ),
        ("Ratio".to_owned(), PropValue::Float(0.5)),
        ("Delta".to_owned(), PropValue::Int(-3)),
    ])
}

#[test]
fn a_table_renders_one_sorted_line_a_key() {
    let text = render_props(&table());
    let want = "Bytes (Read) = 5888208695296\nDelta = -3\nIs Charging = false\n\
                Power Source State = \"AC Power\"\nRatio = 0.5\n";
    assert_eq!(text, want);
}

#[test]
fn rendered_text_parses_back() {
    assert_eq!(parse_props(&render_props(&table())), Some(table()));
    assert_eq!(parse_props(""), Some(Props::new()));
}

#[test]
fn a_whole_float_keeps_its_point() {
    let props = Props::from([("x".to_owned(), PropValue::Float(2.0))]);
    assert_eq!(render_props(&props), "x = 2.0\n");
    assert_eq!(parse_props("x = 2.0\n"), Some(props));
}

#[test]
fn an_unreadable_line_is_none() {
    assert_eq!(parse_props("no separator\n"), None);
    assert_eq!(parse_props("x = maybe\n"), None);
    assert_eq!(parse_props("x = \"open\n"), None);
}

#[test]
fn an_int_is_a_number_of_every_width() {
    let int = PropValue::Int(7);
    assert_eq!(int.as_i64(), Some(7));
    assert_eq!(int.as_u64(), Some(7));
    assert_eq!(int.as_f64(), Some(7.0));
    assert_eq!(PropValue::Int(-1).as_u64(), None);
}

#[test]
fn an_int_is_not_a_bool_or_string() {
    assert_eq!(PropValue::Int(7).as_bool(), None);
    assert_eq!(PropValue::Int(7).as_str(), None);
}

#[test]
fn each_other_kind_reads_as_itself() {
    assert_eq!(PropValue::Float(1.5).as_f64(), Some(1.5));
    assert_eq!(PropValue::Float(1.5).as_i64(), None);
    assert_eq!(PropValue::Bool(true).as_bool(), Some(true));
    assert_eq!(PropValue::Text("a".into()).as_str(), Some("a"));
}

#[test]
fn other_kinds_are_not_numbers() {
    assert_eq!(PropValue::Bool(true).as_f64(), None);
    assert_eq!(PropValue::Text("1".into()).as_i64(), None);
    assert_eq!(PropValue::Float(1.0).as_bool(), None);
    assert_eq!(PropValue::Float(1.0).as_str(), None);
}
