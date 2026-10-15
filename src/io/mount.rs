//! Which block device holds a path: Linux `/proc/self/mountinfo` matched
//! against `/proc/diskstats` (src/io:T2), macOS the `statfs` mount source
//! (src/io:T3).

use std::path::{Path, PathBuf};

use super::{DiskStat, IoError};
use crate::sys::MountSource;

/// `/proc/self/mountinfo`: this process's mounts, one per line.
pub(super) const MOUNTINFO: &str = "/proc/self/mountinfo";

/// One line of `/proc/self/mountinfo`: the columns device resolution
/// reads.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Mount {
    /// Device major number (`0` for a filesystem with no device).
    pub major: u32,
    /// Device minor number.
    pub minor: u32,
    /// Where it is mounted, octal escapes (`\040`) decoded.
    pub mount_point: PathBuf,
    /// Filesystem type, such as `ext4` or `tmpfs`.
    pub fs_type: String,
    /// Mount source, such as `/dev/sda1` or `server:/export`.
    pub source: String,
}

/// Parses `/proc/self/mountinfo`. Blank lines are skipped.
///
/// # Errors
///
/// [`IoError::Malformed`] for a line without `major:minor`, a mount
/// point, the `-` separator, a filesystem type and a source.
pub fn parse_mountinfo(text: &str) -> Result<Vec<Mount>, IoError> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            parse_line(line).ok_or_else(|| IoError::malformed(MOUNTINFO, line))
        })
        .collect()
}

/// One line: id, parent, `major:minor`, root, mount point, options,
/// optional fields up to `-`, filesystem type, source.
fn parse_line(line: &str) -> Option<Mount> {
    let mut words = line.split_whitespace();
    let (major, minor) = words.nth(2)?.split_once(':')?;
    let mount_point = PathBuf::from(unescape(words.nth(1)?));
    let mut tail = words.skip_while(|w| *w != "-").skip(1);
    Some(Mount {
        major: major.parse().ok()?,
        minor: minor.parse().ok()?,
        mount_point,
        fs_type: tail.next()?.to_owned(),
        source: unescape(tail.next()?),
    })
}

/// `field` with the kernel's `\ooo` octal escapes decoded.
fn unescape(field: &str) -> String {
    let mut out = String::with_capacity(field.len());
    let mut rest = field;
    while let Some(at) = rest.find('\\') {
        let (head, tail) = rest.split_at(at);
        out.push_str(head);
        let code = tail.get(1..4).and_then(|o| u8::from_str_radix(o, 8).ok());
        match code {
            Some(byte) => out.push(char::from(byte)),
            None => out.push('\\'),
        }
        let skip = if code.is_some() { 4 } else { 1 };
        rest = tail.get(skip..).unwrap_or_default();
    }
    out.push_str(rest);
    out
}

/// The mount holding absolute `path`: the deepest mount point it is
/// under, the last mounted when two share one (src/io:V1).
#[must_use]
pub fn mount_of<'m>(mounts: &'m [Mount], path: &Path) -> Option<&'m Mount> {
    mounts
        .iter()
        .filter(|m| path.starts_with(&m.mount_point))
        .max_by_key(|m| m.mount_point.components().count())
}

/// The `/proc/diskstats` name of the block device holding absolute
/// `path` (src/io:V1): the device whose `major:minor` the mount shows,
/// else the one its `/dev/` source names (btrfs shows an anonymous
/// `0:N`).
///
/// # Errors
///
/// [`IoError::NoMount`] when no mount holds `path`;
/// [`IoError::NotBlock`] when its filesystem has no counted block
/// device: tmpfs, NFS, FUSE (src/io:V3).
pub fn linux_device(
    mounts: &[Mount],
    disks: &[DiskStat],
    path: &Path,
) -> Result<String, IoError> {
    let mount = mount_of(mounts, path).ok_or_else(|| IoError::NoMount {
        path: path.to_path_buf(),
    })?;
    let by_number =
        |d: &&DiskStat| (d.major, d.minor) == (mount.major, mount.minor);
    let by_source = |d: &&DiskStat| {
        mount.source.strip_prefix("/dev/") == Some(d.name.as_str())
    };
    disks
        .iter()
        .find(by_number)
        .or_else(|| disks.iter().find(by_source))
        .map(|d| d.name.clone())
        .ok_or_else(|| not_block(path, &mount.fs_type))
}

/// The BSD name of the disk holding `path`, from where its filesystem
/// is mounted from: `/dev/disk3s1s1` gives `disk3s1s1`, which
/// `crate::sys::physical_disk_stats` walks up to the physical disk
/// (src/io:V2, R2).
///
/// # Errors
///
/// [`IoError::NotBlock`] when the source is no `/dev/disk*` device:
/// devfs, NFS, SMB, FUSE (src/io:V3).
pub fn macos_device(
    source: &MountSource,
    path: &Path,
) -> Result<String, IoError> {
    source
        .device
        .strip_prefix("/dev/")
        .filter(|name| name.starts_with("disk"))
        .map(str::to_owned)
        .ok_or_else(|| not_block(path, &source.fs_type))
}

/// [`IoError::NotBlock`] for `path` on `fs_type`.
fn not_block(path: &Path, fs_type: &str) -> IoError {
    IoError::NotBlock {
        path: path.to_path_buf(),
        fs_type: fs_type.to_owned(),
    }
}

#[cfg(test)]
mod tests;
