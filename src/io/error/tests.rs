use std::path::PathBuf;

use super::IoError;
use crate::units::parse_name;

#[test]
fn a_malformed_line_renders_trimmed() {
    let malformed = IoError::malformed("/proc/diskstats", " 8 0 sda x \n");
    assert_eq!(
        malformed.to_string(),
        "malformed /proc/diskstats line: `8 0 sda x`"
    );
}

#[test]
fn a_path_without_a_mount_names_it() {
    let no_mount = IoError::NoMount {
        path: PathBuf::from("/x"),
    };
    assert_eq!(no_mount.to_string(), "no mount holds `/x`");
}

#[test]
fn a_mount_without_a_block_device_renders_its_hint() {
    let not_block = IoError::NotBlock {
        path: PathBuf::from("/tmp"),
        fs_type: "tmpfs".to_owned(),
    };
    assert_eq!(
        not_block.to_string(),
        "`/tmp` is on tmpfs, not a block device; name one with --io-device"
    );
}

#[test]
fn an_unknown_device_is_named() {
    let unknown = IoError::UnknownDevice {
        name: "sdz".to_owned(),
    };
    assert_eq!(unknown.to_string(), "no block device `sdz`");
}

#[test]
fn a_bad_device_names_the_switch() {
    let text = parse_name("../sda").err().map(IoError::Device);
    let text = text.map(|e| e.to_string()).unwrap_or_default();
    assert!(text.starts_with("--io-device: `../sda`"), "{text}");
}
