use std::io;
use std::path::{Path, PathBuf};

use super::{
    Mount, linux_device, macos_device, mount_of, parse_mountinfo, unescape,
};
use crate::io::{DiskStat, IoError, parse_diskstats};
use crate::sys::MountSource;

/// The text of fixture file `tests/fixtures/io/<case>/<file>`.
fn fixture(case: &str, file: &str) -> String {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/io");
    std::fs::read_to_string(dir.join(case).join(file)).unwrap_or_default()
}

/// The mounts of fixture `case`.
fn mounts(case: &str) -> Vec<Mount> {
    parse_mountinfo(&fixture(case, "proc/self/mountinfo")).unwrap_or_default()
}

/// The disks of fixture `case`.
fn disks(case: &str) -> Vec<DiskStat> {
    parse_diskstats(&fixture(case, "proc/diskstats")).unwrap_or_default()
}

/// A device lookup that cannot stat anything, as for a path not there.
fn no_stat(_: &Path) -> io::Result<(u32, u32)> {
    Err(io::ErrorKind::NotFound.into())
}

/// The device holding `path` in fixture `case`, by mounts alone.
fn device(case: &str, path: &str) -> Result<String, IoError> {
    linux_device(&mounts(case), &disks(case), Path::new(path), no_stat)
}

/// The device holding `path` in fixture `case` when stat gives `dev`.
fn device_at(
    case: &str,
    path: &str,
    dev: (u32, u32),
) -> Result<String, IoError> {
    let lookup = |_: &Path| Ok(dev);
    linux_device(&mounts(case), &disks(case), Path::new(path), lookup)
}

#[test]
fn a_symlink_across_mounts_resolves_to_its_target_device() {
    // `/tmp/link` sits on tmpfs by prefix, but stat follows it to dm-0.
    assert_eq!(device_at("linux", "/tmp/link", (253, 0)), Ok("dm-0".into()));
}

#[test]
fn the_stat_device_wins_even_under_no_mount() {
    assert_eq!(
        linux_device(&[], &disks("linux"), Path::new("/x"), |_| Ok((8, 0))),
        Ok("sda".to_owned())
    );
}

#[test]
fn an_uncounted_stat_device_falls_back_to_mountinfo() {
    // btrfs: stat shows the anonymous 0:40, the source names sda.
    assert_eq!(device_at("linux", "/data/me", (0, 40)), Ok("sda".into()));
}

#[test]
fn a_mountinfo_line_reads_device_point_type_and_source() {
    let root = mounts("linux").into_iter().next();
    let want = Mount {
        major: 259,
        minor: 1,
        mount_point: PathBuf::from("/"),
        fs_type: "ext4".to_owned(),
        source: "/dev/nvme0n1p1".to_owned(),
    };
    assert_eq!(root, Some(want));
}

#[test]
fn an_escaped_mount_point_is_decoded() {
    let points: Vec<PathBuf> =
        mounts("linux").into_iter().map(|m| m.mount_point).collect();
    assert!(
        points.contains(&PathBuf::from("/srv/my data")),
        "{points:?}"
    );
}

#[test]
fn a_garbled_mountinfo_line_is_malformed() {
    let text = fixture("garbled", "proc/self/mountinfo");
    assert_eq!(
        parse_mountinfo(&text),
        Err(IoError::malformed(
            "/proc/self/mountinfo",
            "not a mountinfo line"
        ))
    );
}

#[test]
fn a_backslash_without_octal_stays() {
    assert_eq!(unescape(r"a\040b\x\1"), r"a b\x\1");
}

#[test]
fn the_deepest_mount_holds_a_path() {
    let all = mounts("linux");
    let point = mount_of(&all, Path::new("/tmp/build/x"))
        .map(|m| m.mount_point.clone());
    assert_eq!(point, Some(PathBuf::from("/tmp")));
}

#[test]
fn a_path_resolves_to_the_device_of_its_mount_number() {
    assert_eq!(device("linux", "/work/proj"), Ok("nvme0n1p1".to_owned()));
    assert_eq!(device("linux", "/srv/my data/x"), Ok("dm-0".to_owned()));
}

#[test]
fn an_anonymous_device_resolves_by_its_source() {
    assert_eq!(device("linux", "/data/me"), Ok("sda".to_owned()));
}

#[test]
fn a_mount_without_a_block_device_is_not_block() {
    let want = |path: &str, fs_type: &str| IoError::NotBlock {
        path: PathBuf::from(path),
        fs_type: fs_type.to_owned(),
    };
    assert_eq!(device("linux", "/tmp/x"), Err(want("/tmp/x", "tmpfs")));
    assert_eq!(device("linux", "/net/a"), Err(want("/net/a", "nfs4")));
}

#[test]
fn a_path_under_no_mount_has_none() {
    assert_eq!(
        linux_device(&[], &[], Path::new("/x"), no_stat),
        Err(IoError::NoMount {
            path: PathBuf::from("/x")
        })
    );
}

/// A macOS mount source of `device` and `fs_type`.
fn source(device: &str, fs_type: &str) -> MountSource {
    MountSource {
        device: device.to_owned(),
        fs_type: fs_type.to_owned(),
    }
}

#[test]
fn a_macos_volume_resolves_to_its_bsd_name() {
    let src = source("/dev/disk3s1s1", "apfs");
    assert_eq!(
        macos_device(&src, Path::new("/")),
        Ok("disk3s1s1".to_owned())
    );
}

#[test]
fn a_macos_mount_without_a_disk_is_not_block() {
    for (device, fs_type) in [("devfs", "devfs"), ("srv:/x", "nfs")] {
        let got = macos_device(&source(device, fs_type), Path::new("/d"));
        let want = IoError::NotBlock {
            path: PathBuf::from("/d"),
            fs_type: fs_type.to_owned(),
        };
        assert_eq!(got, Err(want));
    }
}
