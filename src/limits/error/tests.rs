use super::LimitsError;

#[test]
fn malformed_names_the_file_and_the_trimmed_text() {
    let err = LimitsError::malformed("file-nr", " lots\n");
    assert_eq!(err.to_string(), "malformed file-nr: `lots`");
}
