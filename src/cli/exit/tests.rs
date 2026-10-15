use super::{FAIL, Failure, OK, UNSUPPORTED, USAGE, check_code, slot_failure};
use crate::gate::Overall;
use crate::slot::SlotError;

#[test]
fn failures_carry_their_exit_code() {
    assert_eq!(Failure::usage("bad").code, USAGE);
    let unsupported = Failure::unsupported("no source");
    assert_eq!(unsupported.code, UNSUPPORTED);
    assert_eq!(unsupported.message, "no source");
}

#[test]
fn check_exits_by_the_overall_verdict() {
    assert_eq!(check_code(Overall::Pass), OK);
    assert_eq!(check_code(Overall::Fail), FAIL);
    assert_eq!(check_code(Overall::Warming), FAIL);
    assert_eq!(check_code(Overall::Unsupported), UNSUPPORTED);
}

#[test]
fn a_bad_slot_key_is_usage_and_a_bad_dir_unsupported() {
    let key = SlotError::BadKey {
        input: "..".to_owned(),
    };
    assert_eq!(slot_failure(key).code, USAGE);
    let dir = SlotError::UnsafeDir {
        path: "/tmp/x".into(),
        reason: "is a symlink".to_owned(),
    };
    assert_eq!(slot_failure(dir).code, UNSUPPORTED);
}
