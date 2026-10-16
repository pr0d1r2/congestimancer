use super::splice;

const DOC: &str =
    "# t\n\n<!-- BEGIN x: note -->\nold\n<!-- END x -->\n\nbody\n";

#[test]
fn only_the_named_block_is_replaced() {
    let got = splice(DOC, "x", "NEW\n");
    assert_eq!(got.as_deref(), Some("# t\n\nNEW\n\nbody\n"));
}

#[test]
fn a_block_at_the_end_of_the_file_is_replaced() {
    let end = "<!-- BEGIN x -->\nold\n<!-- END x -->";
    assert_eq!(splice(end, "x", "NEW\n").as_deref(), Some("NEW\n"));
}

#[test]
fn a_missing_or_unclosed_block_is_none() {
    assert_eq!(splice(DOC, "y", "x"), None);
    assert_eq!(splice("<!-- BEGIN x -->\nno end", "x", "x"), None);
}
