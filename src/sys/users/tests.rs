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

#[test]
fn a_small_buffer_grows_until_the_entry_fits() {
    let mut sizes = Vec::new();
    let found = super::with_buffer(|buf| {
        sizes.push(buf.len());
        if buf.len() < 4096 {
            Err(libc::ERANGE)
        } else {
            Ok(Some(7))
        }
    });
    assert_eq!(found.ok(), Some(Some(7)));
    assert_eq!(sizes, vec![1024, 2048, 4096]);
}

#[test]
fn a_buffer_past_the_cap_is_an_error() {
    let found = super::with_buffer(|_| Err::<Option<()>, _>(libc::ERANGE));
    let code = found.err().and_then(|e| e.raw_os_error());
    assert_eq!(code, Some(libc::ERANGE));
}

#[test]
fn not_found_codes_are_none_and_others_are_errors() {
    for code in [libc::ENOENT, libc::ESRCH, libc::EBADF, libc::EPERM] {
        let found = super::with_buffer(|_| Err::<Option<()>, _>(code));
        assert_eq!(found.ok(), Some(None));
    }
    let found = super::with_buffer(|_| Err::<Option<()>, _>(libc::EIO));
    let code = found.err().and_then(|e| e.raw_os_error());
    assert_eq!(code, Some(libc::EIO));
}

#[test]
fn a_name_not_utf8_is_invalid_data_naming_the_uid() {
    let err = super::utf8_name(501, vec![b'a', 0xff]).err();
    let err = err.map(|e| (e.kind(), e.to_string()));
    assert_eq!(
        err,
        Some((
            io::ErrorKind::InvalidData,
            "uid 501: name is not UTF-8".to_owned()
        ))
    );
    let name = super::utf8_name(0, b"root".to_vec());
    assert_eq!(name.ok().as_deref(), Some("root"));
}
