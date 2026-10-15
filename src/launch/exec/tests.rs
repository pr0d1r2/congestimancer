use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::Command;

use super::{exec_command, exec_failure_code};

#[test]
fn not_found_is_127() {
    assert_eq!(exec_failure_code(&Error::from(ErrorKind::NotFound)), 127);
}

#[test]
fn not_executable_is_126() {
    let denied = Error::from(ErrorKind::PermissionDenied);
    assert_eq!(exec_failure_code(&denied), 126);
    assert_eq!(exec_failure_code(&Error::other("ENOEXEC")), 126);
}

#[test]
fn exec_of_a_missing_command_returns_127() {
    let mut command = Command::new("/nonexistent-cgmr-launch/cmd");
    assert_eq!(exec_command(&mut command), 127);
}

#[test]
fn exec_of_a_missing_command_on_path_returns_127() {
    let mut command = Command::new("cgmr-launch-no-such-command");
    assert_eq!(exec_command(&mut command), 127);
}

#[test]
fn exec_of_a_file_without_the_exec_bit_returns_126() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/launch/not-executable");
    assert_eq!(exec_command(&mut Command::new(file)), 126);
}

#[test]
fn exec_of_a_directory_returns_126() {
    assert_eq!(exec_command(&mut Command::new("/")), 126);
}
