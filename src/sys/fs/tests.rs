use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use rustix::fs::{StatVfs, StatVfsMountFlags};

use rustix::io::FdFlags;

use super::{
    FsStats, nice, set_inheritable, set_nice, statvfs, try_lock_exclusive,
    unlock,
};

fn raw() -> StatVfs {
    StatVfs {
        f_bsize: 1_048_576,
        f_frsize: 4096,
        f_blocks: 1000,
        f_bfree: 300,
        f_bavail: 200,
        f_files: 50,
        f_ffree: 20,
        f_favail: 10,
        f_fsid: 7,
        f_flag: StatVfsMountFlags::empty(),
        f_namemax: 255,
    }
}

#[test]
fn counts_come_from_statvfs_in_fragments() {
    let stats = FsStats::from(raw());
    assert_eq!(
        stats,
        FsStats {
            fragment_size: 4096,
            blocks: 1000,
            blocks_free: 300,
            blocks_available: 200,
            files: 50,
            files_free: 20,
            files_available: 10,
        }
    );
    assert_eq!(stats.bytes_total(), 4_096_000);
    assert_eq!(stats.bytes_available(), 819_200);
}

#[test]
fn byte_totals_saturate() {
    let stats = FsStats::from(StatVfs {
        f_blocks: u64::MAX,
        f_bavail: u64::MAX,
        ..raw()
    });
    assert_eq!(stats.bytes_total(), u64::MAX);
    assert_eq!(stats.bytes_available(), u64::MAX);
}

#[test]
fn statvfs_of_a_missing_path_is_not_found() {
    let err = statvfs(Path::new("/nonexistent/cgmr/statvfs")).err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::NotFound));
}

#[test]
fn statvfs_reads_the_fixture_tree_filesystem() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let stats = statvfs(dir).ok();
    assert!(stats.is_some_and(|s| s.fragment_size > 0));
}

/// A lock file of this test's own, removed by the guard.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let pid = std::process::id();
        Self(std::env::temp_dir().join(format!("cgmr-sys-{name}-{pid}")))
    }

    fn open(&self) -> io::Result<File> {
        OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&self.0)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[test]
fn a_second_open_file_cannot_take_a_held_lock() -> io::Result<()> {
    let scratch = Scratch::new("flock");
    let first = scratch.open()?;
    let second = scratch.open()?;
    assert!(try_lock_exclusive(&first)?);
    assert!(!try_lock_exclusive(&second)?);
    unlock(&first)?;
    assert!(try_lock_exclusive(&second)?);
    unlock(&second)
}

#[test]
fn setting_the_nice_value_it_has_is_allowed() -> io::Result<()> {
    let value = nice()?;
    set_nice(value)?;
    assert_eq!(nice()?, value);
    Ok(())
}

#[test]
fn an_inheritable_file_loses_close_on_exec() -> io::Result<()> {
    let scratch = Scratch::new("cloexec");
    let file = scratch.open()?;
    assert!(rustix::io::fcntl_getfd(&file)?.contains(FdFlags::CLOEXEC));
    set_inheritable(&file)?;
    assert!(!rustix::io::fcntl_getfd(&file)?.contains(FdFlags::CLOEXEC));
    Ok(())
}
