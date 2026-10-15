use std::ffi::OsStr;

use super::{Memory, ProcessInfo, Snapshot};

#[test]
fn a_process_keeps_its_name_uid_and_lineage() {
    let info = ProcessInfo::new(OsStr::new("cargo"), Some(501), 7, Some(1));
    assert_eq!(info.name, "cargo");
    assert_eq!(info.uid, Some(501));
    assert_eq!((info.pid, info.parent), (7, Some(1)));
}

#[cfg(unix)]
#[test]
fn a_name_that_is_not_utf8_is_read_lossily() {
    use std::os::unix::ffi::OsStrExt;

    let info = ProcessInfo::new(OsStr::from_bytes(b"a\xffb"), None, 2, None);
    assert_eq!(info.name, "a\u{fffd}b");
}

#[test]
fn nothing_is_read_before_a_refresh_asks() {
    let snapshot = Snapshot::new();
    assert_eq!(snapshot.memory(), Memory::default());
    assert!(snapshot.processes().is_empty());
}

#[test]
fn host_memory_refresh_reads_memory_only() {
    let mut snapshot = Snapshot::new();
    snapshot.refresh_memory();
    let memory = snapshot.memory();
    assert!(memory.total > 0);
    assert!(memory.available <= memory.total);
    assert!(snapshot.processes().is_empty());
}

#[test]
fn host_process_refresh_finds_this_test() {
    let mut snapshot = Snapshot::new();
    snapshot.refresh_processes();
    let procs = snapshot.processes();
    assert!(procs.len() > 1);
    let me = procs.iter().find(|p| p.pid == std::process::id());
    assert!(me.is_some_and(|p| p.parent.is_some()), "{me:?}");
    assert_eq!(snapshot.memory(), Memory::default());
}

#[test]
fn host_load_average_is_not_negative() {
    let load = Snapshot::load_average();
    assert!(load.one >= 0.0 && load.five >= 0.0 && load.fifteen >= 0.0);
}
