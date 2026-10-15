use std::path::PathBuf;

use super::SlotError;

#[test]
fn an_unsafe_dir_names_the_path_and_the_reason() {
    let err = SlotError::UnsafeDir {
        path: PathBuf::from("/tmp/cgmr-501"),
        reason: "is a symlink".to_owned(),
    };
    assert_eq!(
        err.to_string(),
        "slot dir /tmp/cgmr-501: is a symlink; refusing"
    );
}

#[test]
fn a_bad_key_names_the_input_and_the_form() {
    let err = SlotError::BadKey {
        input: "a/b".to_owned(),
    };
    assert!(
        err.to_string()
            .starts_with("--slot-key \"a/b\": not a safe")
    );
}

#[test]
fn an_io_error_names_the_path() {
    let err = SlotError::Io {
        path: PathBuf::from("/x"),
        detail: "denied".to_owned(),
    };
    assert_eq!(err.to_string(), "slot dir /x: denied");
}
