use std::fs::{self, File};
use std::io;
use std::os::unix::fs::MetadataExt;
use std::time::Duration;

use super::{
    lock_wait, lock_within, open_lock, publish, read_all, reap_if_dead,
    rewrite, suffixes,
};
use crate::slot::scratch::{Scratch, eventually};
use crate::sys::try_lock_exclusive;

#[test]
fn a_lock_file_is_made_0600() -> io::Result<()> {
    let scratch = Scratch::new("file-mode")?;
    let path = scratch.path().join("k.0");
    open_lock(&path)?;
    assert_eq!(fs::metadata(&path)?.mode() & 0o077, 0);
    Ok(())
}

#[test]
fn rewrite_replaces_and_read_all_reads_from_the_start() -> io::Result<()> {
    let scratch = Scratch::new("file-rw")?;
    let mut file = open_lock(&scratch.path().join("log"))?;
    rewrite(&mut file, "longer text")?;
    rewrite(&mut file, "short")?;
    assert_eq!(read_all(&mut file)?, "short");
    Ok(())
}

#[test]
fn lock_wait_takes_a_free_lock() -> io::Result<()> {
    let scratch = Scratch::new("file-wait")?;
    let path = scratch.path().join("seq");
    let file = open_lock(&path)?;
    lock_wait(&file)?;
    assert!(!try_lock_exclusive(&open_lock(&path)?)?);
    Ok(())
}

#[test]
fn a_published_file_is_locked_and_has_its_content() -> io::Result<()> {
    let scratch = Scratch::new("file-publish")?;
    let held = publish(scratch.path(), "r.1", "42")?;
    let path = scratch.path().join("r.1");
    assert_eq!(fs::read_to_string(&path)?, "42");
    assert!(!reap_if_dead(&path)?);
    drop(held);
    assert!(eventually(|| reap_if_dead(&path), |dead| *dead)?);
    assert!(!path.exists());
    assert_eq!(suffixes(scratch.path(), "+tmp.")?, Vec::<String>::new());
    Ok(())
}

#[test]
fn a_missing_file_counts_as_dead() -> io::Result<()> {
    let scratch = Scratch::new("file-gone")?;
    assert!(reap_if_dead(&scratch.path().join("gone"))?);
    Ok(())
}

#[test]
fn suffixes_lists_the_rest_of_matching_names() -> io::Result<()> {
    let scratch = Scratch::new("file-list")?;
    File::create(scratch.path().join("a+ticket.3"))?;
    File::create(scratch.path().join("a+ticket.10"))?;
    File::create(scratch.path().join("b+ticket.1"))?;
    let mut found = suffixes(scratch.path(), "a+ticket.")?;
    found.sort();
    assert_eq!(found, vec!["10".to_owned(), "3".to_owned()]);
    Ok(())
}

#[test]
fn a_lock_held_too_long_times_out() -> io::Result<()> {
    let scratch = Scratch::new("file-timeout")?;
    let path = scratch.path().join("seq");
    let held = open_lock(&path)?;
    lock_wait(&held)?;
    let err = lock_within(&open_lock(&path)?, Duration::from_millis(5)).err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::TimedOut));
    Ok(())
}
