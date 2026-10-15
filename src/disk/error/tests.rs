use std::path::PathBuf;

use super::DiskError;

#[test]
fn a_missing_path_names_the_switch_and_the_path() {
    let err = DiskError::Missing {
        path: PathBuf::from("/no/such"),
    };
    assert_eq!(err.to_string(), "--disk-path /no/such: no such path");
}
