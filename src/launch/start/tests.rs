use std::ffi::{OsStr, OsString};

use super::{Start, command, launch};
use crate::slot::HELD_SLOTS_ENV;

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

#[test]
fn the_command_gets_its_arguments_and_a_clean_environment() {
    let argv = ["make", "-j8"].map(OsString::from);
    let vars = [("CGMR_LOAD", "2")].map(|(k, v)| (k.into(), v.into()));
    let Some(built) = command(&argv, vars, Some("make")) else {
        unreachable!("a command was given");
    };
    assert_eq!(built.get_program(), "make");
    assert_eq!(built.get_args().collect::<Vec<_>>(), ["-j8"]);
    let envs: Vec<_> = built.get_envs().collect();
    let removed = (OsStr::new("CGMR_LOAD"), None);
    let held = (OsStr::new(HELD_SLOTS_ENV), Some(OsStr::new("make")));
    assert_eq!(envs, [held, removed]);
}
