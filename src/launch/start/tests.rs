use std::ffi::OsString;

use super::{Start, launch};

#[test]
fn no_command_exits_0() {
    let mut log = Vec::new();
    assert_eq!(launch(&[], Start::default(), Vec::new(), &mut log), 0);
    assert!(log.is_empty());
}

#[test]
fn a_missing_command_still_exits_127() {
    let argv = [OsString::from("/nonexistent-cgmr-launch/cmd")];
    let mut log = Vec::new();
    assert_eq!(launch(&argv, Start::default(), Vec::new(), &mut log), 127);
}
