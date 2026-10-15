use std::ffi::OsStr;
use std::fs::{self, Permissions};
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::PathBuf;

use super::{DirFacts, LockDir, lock_dir_path, unsafe_reason};
use crate::slot::SlotError;
use crate::slot::scratch::Scratch;
use crate::sys::current_uid;

const MINE: DirFacts = DirFacts {
    symlink: false,
    dir: true,
    owner: 501,
    mode: 0o700,
};

#[test]
fn a_private_dir_of_mine_is_safe() {
    assert_eq!(unsafe_reason(&MINE, 501), None);
}

#[test]
fn a_symlink_is_refused() {
    let facts = DirFacts {
        symlink: true,
        dir: false,
        ..MINE
    };
    assert_eq!(unsafe_reason(&facts, 501), Some("is a symlink".into()));
}

#[test]
fn a_file_is_refused() {
    let facts = DirFacts { dir: false, ..MINE };
    assert_eq!(
        unsafe_reason(&facts, 501),
        Some("is not a directory".into())
    );
}

#[test]
fn another_owner_is_refused() {
    let facts = DirFacts { owner: 0, ..MINE };
    assert_eq!(
        unsafe_reason(&facts, 501),
        Some("owned by uid 0, not 501".into())
    );
}

#[test]
fn a_mode_wider_than_0700_is_refused() {
    let facts = DirFacts {
        mode: 0o40755,
        ..MINE
    };
    assert_eq!(
        unsafe_reason(&facts, 501),
        Some("mode 0755 is wider than 0700".into())
    );
}

#[test]
fn a_narrower_mode_is_safe() {
    let facts = DirFacts {
        mode: 0o40500,
        ..MINE
    };
    assert_eq!(unsafe_reason(&facts, 501), None);
}

fn os(s: &str) -> Option<&OsStr> {
    Some(OsStr::new(s))
}

#[test]
fn the_runtime_dir_comes_first() {
    let path = lock_dir_path(os("/run/user/501"), os("/var/tmp"), 501);
    assert_eq!(path, PathBuf::from("/run/user/501/cgmr"));
}

#[test]
fn tmpdir_comes_next() {
    let path = lock_dir_path(None, os("/var/folders/x/T/"), 501);
    assert_eq!(path, PathBuf::from("/var/folders/x/T/cgmr"));
}

#[test]
fn tmp_with_the_uid_comes_last_and_empty_counts_as_unset() {
    assert_eq!(
        lock_dir_path(None, None, 501),
        PathBuf::from("/tmp/cgmr-501")
    );
    assert_eq!(
        lock_dir_path(os(""), os(""), 7),
        PathBuf::from("/tmp/cgmr-7")
    );
}

#[test]
fn open_makes_the_dir_0700() -> io::Result<()> {
    let scratch = Scratch::new("dir-make")?;
    let path = scratch.path().join("cgmr");
    let dir = LockDir::open(path.clone(), current_uid());
    assert_eq!(dir.map(|d| d.path().to_owned()).ok(), Some(path.clone()));
    let mode = fs::metadata(&path)?.mode() & 0o777;
    assert_eq!(mode & 0o077, 0);
    Ok(())
}

#[test]
fn open_accepts_an_existing_private_dir() -> io::Result<()> {
    let scratch = Scratch::new("dir-again")?;
    let path = scratch.path().to_owned();
    assert!(LockDir::open(path.clone(), current_uid()).is_ok());
    assert!(LockDir::open(path, current_uid()).is_ok());
    Ok(())
}

#[test]
fn open_refuses_a_wide_dir() -> io::Result<()> {
    let scratch = Scratch::new("dir-wide")?;
    fs::set_permissions(scratch.path(), Permissions::from_mode(0o755))?;
    let err = LockDir::open(scratch.path().to_owned(), current_uid()).err();
    assert!(matches!(err, Some(SlotError::UnsafeDir { .. })));
    Ok(())
}

#[test]
fn open_refuses_a_symlink() -> io::Result<()> {
    let scratch = Scratch::new("dir-link")?;
    let link = scratch.path().join("link");
    symlink(scratch.path(), &link)?;
    let err = LockDir::open(link, current_uid()).err();
    assert!(matches!(
        err,
        Some(SlotError::UnsafeDir { reason, .. }) if reason == "is a symlink"
    ));
    Ok(())
}

#[test]
fn open_refuses_another_uid() -> io::Result<()> {
    let scratch = Scratch::new("dir-owner")?;
    let other = current_uid().wrapping_add(1);
    let err = LockDir::open(scratch.path().to_owned(), other).err();
    assert!(matches!(err, Some(SlotError::UnsafeDir { .. })));
    Ok(())
}

#[test]
fn open_reports_a_dir_it_cannot_make() {
    let path = PathBuf::from("/nonexistent/cgmr-slot/cgmr");
    let err = LockDir::open(path, current_uid()).err();
    assert!(matches!(err, Some(SlotError::Io { .. })));
}
