use std::io;
use std::path::Path;

use super::{list_dir, read_text, utf8_name};
use crate::signal::Root;

/// This node's fixture tree, laid out like `/`.
fn fixtures() -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/sys/root"))
}

#[test]
fn reads_a_file_under_the_root() {
    let text = read_text(&fixtures(), "/proc/sys/fs/file-nr");
    assert_eq!(text.ok().as_deref(), Some("2304\t0\t9223372036854775807\n"));
}

#[test]
fn a_missing_file_is_unsupported_and_names_the_path() {
    let err = read_text(&fixtures(), "/proc/pressure/cpu").err();
    let err = err.map(|e| (e.kind(), e.to_string()));
    assert_eq!(
        err,
        Some((
            io::ErrorKind::Unsupported,
            "/proc/pressure/cpu: not present".to_owned()
        ))
    );
}

#[test]
fn other_failures_keep_their_kind() {
    let err = read_text(&fixtures(), "/proc/sys").err();
    let kind = err.map(|e| e.kind());
    assert!(kind.is_some());
    assert_ne!(kind, Some(io::ErrorKind::Unsupported));
}

#[test]
fn lists_a_directory_sorted() {
    let names = list_dir(&fixtures(), "/sys/class/power_supply");
    assert_eq!(names.ok(), Some(vec!["AC".to_owned(), "BAT0".to_owned()]));
}

#[test]
fn lists_files_as_well_as_directories() {
    let names = list_dir(&fixtures(), "/proc/sys/fs");
    assert_eq!(names.ok(), Some(vec!["file-nr".to_owned()]));
}

#[test]
fn a_missing_directory_is_unsupported_and_names_the_path() {
    let err = list_dir(&fixtures(), "/sys/class/net").err();
    let err = err.map(|e| (e.kind(), e.to_string()));
    assert_eq!(
        err,
        Some((
            io::ErrorKind::Unsupported,
            "/sys/class/net: not present".to_owned()
        ))
    );
}

#[test]
fn listing_a_file_keeps_its_own_error_kind() {
    let err = list_dir(&fixtures(), "/proc/sys/fs/file-nr").err();
    let kind = err.map(|e| e.kind());
    assert!(kind.is_some());
    assert_ne!(kind, Some(io::ErrorKind::Unsupported));
}

#[test]
fn a_name_not_utf8_is_invalid_data_naming_the_directory() {
    use std::os::unix::ffi::OsStringExt as _;
    let name = std::ffi::OsString::from_vec(vec![b'a', 0xff]);
    let err = utf8_name("/proc", name).err();
    let err = err.map(|e| (e.kind(), e.to_string()));
    assert_eq!(
        err,
        Some((
            io::ErrorKind::InvalidData,
            "/proc: entry a\u{FFFD} is not UTF-8".to_owned()
        ))
    );
}
