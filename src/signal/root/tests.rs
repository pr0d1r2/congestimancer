use std::path::{Path, PathBuf};

use super::Root;

/// This node's fixture tree, laid out like `/`.
fn fixtures() -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/signal/root"))
}

#[test]
fn the_system_root_is_slash() {
    let root = Root::system();
    assert_eq!(root.path(), Path::new("/"));
    assert_eq!(root.join("/proc/loadavg"), Path::new("/proc/loadavg"));
    assert_eq!(Root::default(), root);
}

#[test]
fn a_fixture_root_stands_for_slash() {
    let root = Root::new("fx");
    assert_eq!(root.path(), Path::new("fx"));
    assert_eq!(root.join("/proc/loadavg"), Path::new("fx/proc/loadavg"));
    assert_eq!(root.join("proc/loadavg"), Path::new("fx/proc/loadavg"));
    assert_eq!(root.join("/"), Path::new("fx"));
}

#[test]
fn dots_never_climb_above_the_root() {
    let root = Root::new("fx");
    let cases = [
        ("/sys/../../../etc/passwd", "fx/etc/passwd"),
        ("/a/./b/../c", "fx/a/c"),
        ("../..", "fx"),
    ];
    for (path, want) in cases {
        assert_eq!(root.join(path), PathBuf::from(want), "{path}");
    }
}

#[test]
fn a_fixture_file_reads_through_the_root() {
    let path = fixtures().join("/proc/loadavg");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(text.starts_with("0.52 0.58 0.59"), "{path:?}: {text:?}");
}
