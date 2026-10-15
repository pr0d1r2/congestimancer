use std::io;
use std::path::Path;

use super::read_text;
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
