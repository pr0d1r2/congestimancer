use std::io;
use std::os::unix::fs::MetadataExt as _;

use super::{current_uid, user_id, user_name};

#[test]
fn root_is_uid_zero() {
    assert_eq!(user_id("root").ok(), Some(Some(0)));
    assert_eq!(user_name(0).ok(), Some(Some("root".to_owned())));
}

#[test]
fn an_unknown_user_is_none() {
    assert_eq!(user_id("cgmr-no-such-user").ok(), Some(None));
    assert_eq!(user_name(4_000_000_123).ok(), Some(None));
}

#[test]
fn a_name_with_a_nul_is_invalid_input() {
    let kind = user_id("ro\0ot").err().map(|e| e.kind());
    assert_eq!(kind, Some(io::ErrorKind::InvalidInput));
}

#[test]
fn current_uid_owns_the_files_this_process_creates() {
    let path = std::env::temp_dir()
        .join(format!("cgmr-sys-users-{}", std::process::id()));
    let made = std::fs::write(&path, b"");
    let owner = std::fs::metadata(&path).map(|m| m.uid());
    let _ = std::fs::remove_file(&path);
    assert!(made.is_ok());
    assert_eq!(owner.ok(), Some(current_uid()));
}

#[test]
fn the_current_user_round_trips_when_named() {
    let uid = current_uid();
    if let Ok(Some(name)) = user_name(uid) {
        assert_eq!(user_id(&name).ok(), Some(Some(uid)));
    }
}
