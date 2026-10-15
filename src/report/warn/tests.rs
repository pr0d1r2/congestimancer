use super::Warnings;

fn written(quiet: bool, texts: &[&str]) -> String {
    let mut warnings = Warnings::new(Vec::new(), quiet);
    for text in texts {
        assert!(warnings.warn(text).is_ok());
    }
    String::from_utf8(warnings.into_inner()).unwrap_or_default()
}

#[test]
fn a_warning_is_one_prefixed_line() {
    assert_eq!(written(false, &["slow disk"]), "warning: slow disk\n");
}

#[test]
fn a_prefixed_warning_keeps_one_prefix() {
    let text = "warning: use --cpu-pressure instead";
    assert_eq!(written(false, &[text]), format!("{text}\n"));
}

#[test]
fn quiet_writes_nothing() {
    assert_eq!(written(true, &["a", "warning: b"]), "");
}

#[test]
fn warnings_follow_one_another() {
    let want = "warning: a\nwarning: b\n";
    assert_eq!(written(false, &["a", "b"]), want);
}
