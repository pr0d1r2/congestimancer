use super::PowerError;

#[test]
fn malformed_names_the_file_and_the_trimmed_text() {
    let err = PowerError::malformed("temp", " hot\n");
    assert_eq!(err.to_string(), "malformed temp: `hot`");
}
